//! Session Module — Cell-based 会话系统
//!
//! Cell 是划分 session 上下文的单元（类似 Jupyter Notebook 的代码块）。
//! 每个 Cell 内部持有 session，封装消息存取、结果管理、生命周期。
//!
//! 两种 Cell 模式:
//! - InMemoryCell: Summarizer 专用，纯内存
//! - JsonlCell: Builder/Executor/Optimizer 专用，JSONL 持久化
//!
//! CellManager 替代原 SessionManager，统一管理所有 Cell。

pub mod types;
pub mod cell;

pub use types::Message;
pub use cell::{
    CellInfo, CellStatus, SessionCell, manager::CellManager,
    result::{ResultStatus, StoredResult},
};
