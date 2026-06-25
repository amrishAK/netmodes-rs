
use std::marker::PhantomData;

use uuid::Uuid;

use crate::core::socket::Socket;

pub struct TcpServerConfiguration {
    pub host: String,
    pub port: u16
}

pub struct TcpServer {
    pub config: TcpServerConfiguration,
    pub id: Uuid,
    pub(crate) fd: Socket,
}

pub struct TcpClient{
    pub id: Uuid,
    pub(crate) fd: Socket,
}

pub struct TcpServerHandler<State> {
    pub server: TcpServer,
    pub(crate) _state: PhantomData<State>,
}