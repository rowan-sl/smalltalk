use std::{fmt::Debug, net::SocketAddr, ops::{Deref, DerefMut}};

use serde::{de::DeserializeOwned, Serialize};
use tokio::net::TcpStream;

use crate::socket::interface::SocketUtils;

pub mod error {
    use std::fmt::Debug;

    #[derive(Debug, thiserror::Error)]
    #[error("Failed to connect!\n{err}")]
    pub struct ConnectError {
        #[from]
        #[source]
        err: std::io::Error,
    }
}


/// A basic interface to represent a client,
/// with methods for sending and reiving normal rust types
pub struct Client<H, M, O>
where
    H: crate::header::IsHeader,
    M: Serialize + DeserializeOwned,
    O: bincode::Options + Clone,
{
    sock_interface: SocketUtils<H, M, O>
}

impl<H, M, O> Client<H, M, O>
where
    H: crate::header::IsHeader + Clone + Debug,
    M: Serialize + DeserializeOwned,
    O: bincode::Options + Clone,
{
    /// Creates a new [`Client`], connecting to `addr`
    ///
    /// # Args
    /// `bincode_opts` is used for serializing and deserializing messages, see [`Options`] for more info
    ///
    /// [`Options`]: bincode::Options
    pub async fn connect(addr: SocketAddr, bincode_opts: O) -> Result<Self, error::ConnectError> {
        let (read_half, write_half) =
            crate::socket::split_stream(TcpStream::connect(addr).await?, bincode_opts);
        let sock_interface = SocketUtils::new(read_half, write_half, addr);
        Ok(Self {
            sock_interface,
        })
    }
}

impl<H, M, O> Deref for Client<H, M, O>
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

impl<H, M, O> DerefMut for Client<H, M, O>
where
    H: crate::header::IsHeader + Debug + Clone,
    M: Serialize + DeserializeOwned,
    O: bincode::Options + Clone,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.sock_interface
    }
}
