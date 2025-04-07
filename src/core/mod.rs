//! 核心数据结构模块

pub mod session;
pub mod tool;

pub use session::{SessionData, SessionStatus, Message, Role, Config};
pub use tool::{Tool, ToolError, ToolInput, ToolOutput, ToolRegistry};