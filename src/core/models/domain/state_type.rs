/// Lifecycle states for a TCP server instance.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TcpServerState {
	/// Server is not yet created from user settings.
	Unconfigured,
	/// Server instance was created but not yet initialized.
	Created,
	/// Socket is bound to host and port.
	Bound,
	/// Socket is actively listening for new connections.
	Listening,
	/// Socket has been closed and cannot be reused.
	Closed,
}


/// Marker type for a server in the unconfigured state.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Unconfigured;

/// Marker type for a server in the created state.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Created;

/// Marker type for a server in the bound state.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Bound;

/// Marker type for a server in the listening state.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Listening;

/// Marker type for a server in the closed state.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Closed;