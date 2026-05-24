//! Session Module — Cell-based 会话系统
//!
//! Cell 是划分 session 上下文的单元（类似 Jupyter Notebook 的代码块）。
//! 每个 Cell 内部持有 session，封装消息存取、结果管理、生命周期。
//!
//! 分层结构：
//! - session/：Session trait + InMemorySession / JsonlSession（持久化策略）
//! - cell/：SessionCell trait + 具体 Cell structs（行为绑定）
//! - manager.rs：CellManager（生命周期管理）

pub mod types;
pub mod session;
pub mod cell;
pub mod manager;

pub use types::{EffectType, SessionLine};
pub use cell::{
    CellInfo, 
    CellStatus, 
    CellType, 
    SessionCell,
};
pub use manager::CellManager;
