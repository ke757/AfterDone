use std::collections::VecDeque;

use crate::error::{AppError, AppResult};
use crate::llm::events::{
    ContentBlockHeader, ContentDelta, StreamEvent, StreamEventStream,
};
use crate::llm::message::{AssistantContentBlock, AssistantMessage, ToolCallContent};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Reducer — rig MultiTurnStreamItem → StreamEvent 状态机
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[derive(Debug, Clone)]
pub(crate) enum ReducerState {
    Idle,
    InTextBlock { index: usize },
    InThinkingBlock { index: usize },
    InToolCallBlock { index: usize, internal_call_id: String },
    Complete,
}

impl ReducerState {
    fn block_index(&self) -> Option<usize> {
        match self {
            ReducerState::InTextBlock { index }
            | ReducerState::InThinkingBlock { index }
            | ReducerState::InToolCallBlock { index, .. } => Some(*index),
            _ => None,
        }
    }

    fn next_block_index(&self) -> usize {
        match self {
            ReducerState::InTextBlock { index }
            | ReducerState::InThinkingBlock { index }
            | ReducerState::InToolCallBlock { index, .. } => index + 1,
            _ => 0,
        }
    }
}

pub(crate) struct Reducer {
    pub(crate) state: ReducerState,
    pub(crate) pending: VecDeque<StreamEvent>,
    pub(crate) blocks: Vec<AssistantContentBlock>,
}

impl Reducer {
    pub(crate) fn new() -> Self {
        Self {
            state: ReducerState::Idle,
            pending: VecDeque::new(),
            blocks: Vec::new(),
        }
    }

    fn stop_current_block(&mut self) {
        if let Some(index) = self.state.block_index() {
            self.pending
                .push_back(StreamEvent::ContentBlockStop { index });
        }
    }

    pub(crate) fn feed(
        &mut self,
        chunk: rig::streaming::StreamedAssistantContent<impl Clone>,
    ) {
        use rig::streaming::StreamedAssistantContent;

        match chunk {
            StreamedAssistantContent::Text(text) => {
                let text_str = text.to_string();
                match &self.state {
                    ReducerState::InTextBlock { index } => {
                        self.pending
                            .push_back(StreamEvent::ContentBlockDelta {
                                index: *index,
                                delta: ContentDelta::Text(text_str),
                            });
                    }
                    _ => {
                        self.stop_current_block();
                        let next_idx = self.state.next_block_index();
                        self.state = ReducerState::InTextBlock { index: next_idx };
                        self.pending
                            .push_back(StreamEvent::ContentBlockStart {
                                index: next_idx,
                                block: ContentBlockHeader::Text,
                            });
                        self.pending
                            .push_back(StreamEvent::ContentBlockDelta {
                                index: next_idx,
                                delta: ContentDelta::Text(text_str),
                            });
                    }
                }
            }

            StreamedAssistantContent::ReasoningDelta {
                id: _id,
                reasoning,
            } => match &self.state {
                ReducerState::InThinkingBlock { index } => {
                    self.pending
                        .push_back(StreamEvent::ContentBlockDelta {
                            index: *index,
                            delta: ContentDelta::Thinking(reasoning),
                        });
                }
                _ => {
                    self.stop_current_block();
                    let next_idx = self.state.next_block_index();
                    self.state = ReducerState::InThinkingBlock { index: next_idx };
                    self.pending
                        .push_back(StreamEvent::ContentBlockStart {
                            index: next_idx,
                            block: ContentBlockHeader::Thinking,
                        });
                    self.pending
                        .push_back(StreamEvent::ContentBlockDelta {
                            index: next_idx,
                            delta: ContentDelta::Thinking(reasoning),
                        });
                }
            },

            StreamedAssistantContent::Reasoning(reasoning) => {
                self.stop_current_block();
                let next_idx = self.state.next_block_index();
                self.state = ReducerState::Idle;

                self.pending
                    .push_back(StreamEvent::ContentBlockStart {
                        index: next_idx,
                        block: ContentBlockHeader::Thinking,
                    });

                for content in &reasoning.content {
                    let text = match content {
                        rig::message::ReasoningContent::Text {
                            text,
                            signature: _,
                        } => text.clone(),
                        rig::message::ReasoningContent::Summary(text) => text.clone(),
                        _ => continue,
                    };
                    self.pending
                        .push_back(StreamEvent::ContentBlockDelta {
                            index: next_idx,
                            delta: ContentDelta::Thinking(text),
                        });
                }

                self.pending
                    .push_back(StreamEvent::ContentBlockStop { index: next_idx });
            }

            StreamedAssistantContent::ToolCallDelta {
                id,
                internal_call_id,
                content,
            } => {
                let same_block = matches!(
                    &self.state,
                    ReducerState::InToolCallBlock {
                        internal_call_id: current_id,
                        ..
                    } if *current_id == internal_call_id
                );

                if same_block {
                    let idx = self.state.block_index().unwrap();
                    match content {
                        rig::streaming::ToolCallDeltaContent::Name(name) => {
                            self.pending.push_back(StreamEvent::ContentBlockDelta {
                                index: idx,
                                delta: ContentDelta::ToolCallName(name),
                            });
                        }
                        rig::streaming::ToolCallDeltaContent::Delta(args) => {
                            self.pending.push_back(StreamEvent::ContentBlockDelta {
                                index: idx,
                                delta: ContentDelta::ToolCallArguments(args),
                            });
                        }
                    }
                } else {
                    self.stop_current_block();
                    let next_idx = self.state.next_block_index();
                    self.state = ReducerState::InToolCallBlock {
                        index: next_idx,
                        internal_call_id: internal_call_id.clone(),
                    };
                    self.pending
                        .push_back(StreamEvent::ContentBlockStart {
                            index: next_idx,
                            block: ContentBlockHeader::ToolCall {
                                id: id.clone(),
                                name: None,
                            },
                        });
                    match content {
                        rig::streaming::ToolCallDeltaContent::Name(name) => {
                            self.pending.push_back(StreamEvent::ContentBlockDelta {
                                index: next_idx,
                                delta: ContentDelta::ToolCallName(name),
                            });
                        }
                        rig::streaming::ToolCallDeltaContent::Delta(args) => {
                            self.pending.push_back(StreamEvent::ContentBlockDelta {
                                index: next_idx,
                                delta: ContentDelta::ToolCallArguments(args),
                            });
                        }
                    }
                }
            }

            StreamedAssistantContent::ToolCall {
                tool_call,
                internal_call_id: _,
            } => {
                self.stop_current_block();
                let next_idx = self.state.next_block_index();
                let id = tool_call.id.clone();
                let name = tool_call.function.name.clone();
                let args = tool_call.function.arguments.clone();

                self.pending
                    .push_back(StreamEvent::ContentBlockStart {
                        index: next_idx,
                        block: ContentBlockHeader::ToolCall {
                            id: id.clone(),
                            name: Some(name.clone()),
                        },
                    });
                self.pending
                    .push_back(StreamEvent::ContentBlockDelta {
                        index: next_idx,
                        delta: ContentDelta::ToolCallArguments(
                            serde_json::to_string(&args).unwrap_or_default(),
                        ),
                    });
                self.pending
                    .push_back(StreamEvent::ContentBlockStop { index: next_idx });
                self.blocks
                    .push(AssistantContentBlock::ToolCall(ToolCallContent {
                        id,
                        name,
                        arguments: args,
                    }));
                self.state = ReducerState::Idle;
            }

            StreamedAssistantContent::Final(_final) => {
                self.stop_current_block();
                self.state = ReducerState::Complete;

                let message = AssistantMessage {
                    content: std::mem::take(&mut self.blocks),
                };
                self.pending
                    .push_back(StreamEvent::MessageComplete { message });
            }
        }
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// adapt_rig_stream — 将 rig 多轮流转换为我们的 StreamEventStream
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

pub fn adapt_rig_stream<R>(
    inner: std::pin::Pin<
        Box<
            dyn futures::Stream<
                    Item = Result<
                        rig::agent::MultiTurnStreamItem<R>,
                        rig::agent::StreamingError,
                    >,
                > + Send,
        >,
    >,
) -> StreamEventStream
where
    R: Clone + Unpin + Send + 'static,
{
    use async_stream::stream;
    use futures::StreamExt;
    use rig::agent::MultiTurnStreamItem;

    let mut inner = inner;

    Box::pin(stream! {
        let mut reducer = Reducer::new();

        loop {
            while let Some(event) = reducer.pending.pop_front() {
                yield Ok(event);
            }

            if matches!(reducer.state, ReducerState::Complete) {
                return;
            }

            match inner.next().await {
                Some(Ok(MultiTurnStreamItem::StreamAssistantItem(content))) => {
                    reducer.feed(content);
                }
                Some(Ok(MultiTurnStreamItem::FinalResponse(final_resp))) => {
                    reducer.stop_current_block();
                    reducer.state = ReducerState::Complete;

                    let text = final_resp.response().to_string();
                    let message = if text.is_empty() && !reducer.blocks.is_empty() {
                        AssistantMessage {
                            content: std::mem::take(&mut reducer.blocks),
                        }
                    } else {
                        AssistantMessage::text_only(&text)
                    };
                    yield Ok(StreamEvent::MessageComplete { message });
                }
                Some(Ok(MultiTurnStreamItem::StreamUserItem(_))) => {
                    // Tool results sent by user — silently skip
                }
                Some(Ok(_other)) => {
                    // Non-exhaustive enum — silently skip unknown variants
                }
                Some(Err(e)) => {
                    yield Err(AppError::Llm(format!("Stream error: {}", e)));
                    return;
                }
                None => {
                    if !matches!(reducer.state, ReducerState::Complete) {
                        reducer.stop_current_block();
                        let message = AssistantMessage {
                            content: std::mem::take(&mut reducer.blocks),
                        };
                        yield Ok(StreamEvent::MessageComplete { message });
                    }
                    return;
                }
            }
        }
    })
}
