pub mod types;
pub mod traits;
pub mod supervisor;
pub mod summarizer;
// pub mod executor;    // Phase 5
// pub mod optimizer;   // Phase 6

pub use types::*;
pub use traits::SideCarAgent;
pub use supervisor::AgentSupervisor;
pub use summarizer::GoalSummarizerAgent;
