use std::net::SocketAddr;
use std::fmt::Debug;
use std::ops::{Deref, DerefMut};

use serde::{de::DeserializeOwned, Serialize};
use tokio::net::{TcpListener, ToSocketAddrs};

use crate::socket::{self, interface::SocketUtils};

pub mod error {
    #[derive(thiserror::Error, Debug)]
    pub enum BindServerError {
        #[error("Failed to bind listener!\n{0}")]
        ListenerBindError(#[from] std::io::Error),
    }

    #[derive(thiserror::Error, Debug)]
    #[error("Error while accpeting a connection!")]
    pub struct AcceptConnectionError {
        #[source]
        #[from]
        source: std::io::Error,
    }
}

/// A connection to a client.
/// 
/// produced by the servers accept method
pub struct ClientConnection<H, M, O>
where
    H: crate::header::IsHeader + Debug + Clone,
    M: Serialize + DeserializeOwned,
    O: bincode::Options + Clone,
{
    sock_interface: SocketUtils<H, M, O>
}

impl<H, M, O> ClientConnection<H, M, O>
where
    H: crate::header::IsHeader + Debug + Clone,
    M: Serialize + DeserializeOwned,
    O: bincode::Options + Clone,
{
    pub(crate) fn new(addr: SocketAddr, reader: socket::Reader<H, M, O>, writer: socket::Writer<H, M, O>) -> Self {
        Self {
            sock_interface: SocketUtils::new(reader, writer, addr)
        }
    }
}

impl<H, M, O> Deref for ClientConnection<H, M, O>
where
    H: crate::header::IsHeader + Debug + Clone,
    M: Serialize + DeserializeOwned,
    O: bincode::Options + Clone,
{
    type Target = SocketUtils<H, M, O>;
    fn deref(&self) -> &Self::Target {
        &self.sock_interface
    }
}

impl<H, M, O> DerefMut for ClientConnection<H, M, O>
where
    H: crate::header::IsHeader + Debug + Clone,
    M: Serialize + DeserializeOwned,
    O: bincode::Options + Clone,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.sock_interface
    }
}

/// A Server wrapping a TcpListener,
/// with utils for accepting new clients.
pub struct Server<O>
where
    O: bincode::Options + Clone,
{
    listener: TcpListener,
    bincode_options: O,
}

impl<O> Server<O>
where
    O: bincode::Options + Clone + Send,
{
    /// Binds the server to the provided adress.
    /// The server does not listen for new connections imediataly, for that you need `.listen()`
    ///
    /// The provided callback will be used when running the server,
    /// it is run in a new task, so it can be asynchronously blocking, although should not be non async blocking
    ///
    /// # Errors
    /// if it could not sucessfully bind to the provided adress
    pub async fn bind<A: ToSocketAddrs>(
        addr: A,
        bincode_opts: O,
    ) -> Result<Server<O>, error::BindServerError> {
        let listener = TcpListener::bind(addr).await?;
        Ok(Self {
            listener,
            bincode_options: bincode_opts,
        })
    }

    pub async fn accept<H, M>(
        &mut self,
    ) -> Result<
        ClientConnection<H, M, O>,
        error::AcceptConnectionError,
    >
    where
        H: crate::header::IsHeader + Clone + Send + Debug,
        M: Serialize + DeserializeOwned + Send,
    {
        let conn = self.listener.accept().await?;
        let (read_half, write_half) = socket::split_stream(conn.0, self.bincode_options.clone());
        Ok(ClientConnection::new(conn.1, read_half, write_half))
    }

    pub fn as_listener(&self) -> &TcpListener {
        &self.listener
    }

    pub fn as_listener_mut(&mut self) -> &mut TcpListener {
        &mut self.listener
    }

    pub fn into_listener(self) -> TcpListener {
        self.listener
    }
}
