pub mod core;
pub mod tcp_socket_handler;

// Namespace facade so APIs can be reached via `thugal::net` style paths.
pub mod net {
	pub use crate::core;
	pub use crate::tcp_socket_handler;
}

pub mod thugal {
	pub use crate::net;
}