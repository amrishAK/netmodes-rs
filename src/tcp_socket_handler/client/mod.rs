/// Outbound TCP client lifecycle and I/O operations.
pub mod client_handler;

pub use crate::core::models::domain::state_type::{Connected, Disconnected};
pub use crate::core::models::domain::tcp::{TcpClient, TcpClientConfiguration, TcpClientHandler};
