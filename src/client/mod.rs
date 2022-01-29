use std::{fmt::Debug, net::SocketAddr};

use serde::{de::DeserializeOwned, Serialize};
use tokio::net::TcpStream;

use crate::{Reader, Writer};

pub mod error {
    use std::fmt::Debug;

    #[derive(Debug, thiserror::Error)]
    #[error("Failed to connect!\n{err}")]
    pub struct ConnectError {
        #[from]
        #[source]
        err: std::io::Error,
    }

    #[derive(Debug, thiserror::Error)]
    pub enum UpdateError<H: crate::header::IsHeader + Debug> {
        #[error("Failed to read from socket!\n{0}")]
        Read(std::io::Error),
        #[error("Failed to deserialize message from socket!\n{0}")]
        ReadUpdate(crate::socket::read::error::UpdateError<H>),
        #[error("Failed to write data to socket!\n{0}")]
        Write(crate::socket::write::error::WriteError),
    }

    #[derive(Debug, thiserror::Error)]
    pub enum WaitMessageError<H: crate::header::IsHeader + Debug> {
        #[error("Failed to update client while waiting for a message!\n{0}")]
        Update(#[from] UpdateError<H>),
        #[error("Failed to read from socket while waiting for a message!\n{0}")]
        Read(#[from] std::io::Error),
    }
}

pub mod res {
    #[derive(Debug, Clone)]
    pub struct UpdateClientStatus {
        new_message: bool,
    }

    impl UpdateClientStatus {
        pub fn new(new_message: bool) -> Self {
            Self { new_message }
        }

        pub fn new_msg(&self) -> bool {
            self.new_message
        }
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
    reader: Reader<H, M, O>,
    writer: Writer<H, M, O>,
    addr: SocketAddr,
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
        Ok(Self {
            reader: read_half,
            writer: write_half,
            addr,
        })
    }

    /// Attempt to read some data from the socket,
    /// blocking untill at least a little bit of data has been read
    ///
    /// this is mostly a convenience function for `self.as_reader_mut().read()`
    ///
    /// for more info see [`Reader.read()`]
    ///
    /// [`Reader.read()`]: crate::socket::read::Reader
    pub async fn update_read(&mut self) -> std::io::Result<()> {
        self.reader.read().await
    }

    /// Updates the reader and writer,
    /// deserializing incoming messages if there are any
    /// and writing data to the socket.
    ///
    /// this should *not* take long to finish, as it does not wait for anything
    ///
    /// for more info see [`Reader::update`] and [`Writer::write`]
    ///
    /// # Returns
    /// if sucsesfull, weather or not a new message is ready to be read.
    ///
    /// # Errors
    /// if deserializing a incoming message or writing to the inner [`Writer`] fails
    ///
    /// [`Reader::update`]: crate::socket::read::Reader
    /// [`Writer::write`]: crate::socket::write::Writer
    pub async fn update(&mut self) -> Result<res::UpdateClientStatus, error::UpdateError<H>> {
        let new_message;
        match self.reader.update().await {
            Ok(nm) => {
                new_message = nm;
            }
            Err(e) => return Err(error::UpdateError::ReadUpdate(e)),
        }
        match self.writer.write().await {
            Ok(_) => {}
            Err(e) => return Err(error::UpdateError::Write(e)),
        }
        Ok(res::UpdateClientStatus::new(new_message))
    }

    /// Repeatedly reads from the socket and updates the client,
    /// untill a new message is available, then returns it
    ///
    /// This is mostly a convenice function, but it should be fine to use in real code
    ///
    /// # Panics
    /// it shouldent, so please do tell if it does
    pub async fn wait_for_message(
        &mut self,
    ) -> Result<crate::msg::MessageWrapper<M, H>, error::WaitMessageError<H>> {
        loop {
            self.update_read().await?;
            if self.update().await?.new_msg() {
                if let Some(m) = self.reader.latest_message() {
                    return Ok(m);
                } else {
                    panic!("This should not happen, and if it does please submit a bug report\nSaying that Client::update() incorrectly returned that there was a message when there was not");
                }
            }
        }
    }

    /// Gets all incoming messages that have been received
    pub fn get_messages(&mut self) -> std::vec::Drain<crate::msg::MessageWrapper<M, H>> {
        self.reader.ready_messages()
    }

    /// Gets the latest incoming message received
    pub fn get_latest_message(&mut self) -> Option<crate::msg::MessageWrapper<M, H>> {
        self.reader.latest_message()
    }

    /// Queues a [`Message`] to be sent
    ///
    /// [`Message`]: crate::msg::MessageWrapper
    pub fn queue_message(
        &mut self,
        message: &crate::msg::MessageWrapper<M, H>,
    ) -> Result<(), crate::socket::write::error::SeriError> {
        self.writer.queue(message)
    }

    /// Gets the address the client is connected to
    pub fn addr(&self) -> SocketAddr {
        self.addr
    }

    pub fn as_reader(&self) -> &Reader<H, M, O> {
        &self.reader
    }

    pub fn as_writer(&self) -> &Writer<H, M, O> {
        &self.writer
    }

    pub fn as_reader_mut(&mut self) -> &mut Reader<H, M, O> {
        &mut self.reader
    }

    pub fn as_writer_mut(&mut self) -> &mut Writer<H, M, O> {
        &mut self.writer
    }

    pub fn into_rw(self) -> (Reader<H, M, O>, Writer<H, M, O>) {
        (self.reader, self.writer)
    }
}
