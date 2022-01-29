use std::{collections::VecDeque, marker::PhantomData};

use bytes::{Buf, Bytes};
use serde::{de::DeserializeOwned, Serialize};
use tokio::{io::AsyncWriteExt, net::tcp::OwnedWriteHalf};

pub mod error {
    #[derive(Debug, thiserror::Error)]
    #[error("Failed to serialize message!\n{0}")]
    pub struct SeriError(#[from] bincode::Error);

    #[derive(Debug, thiserror::Error)]
    pub enum WriteError {
        #[error("Error while sending data!\n{0}")]
        IOError(#[from] std::io::Error),
        #[error("Socket Closed!")]
        Disconnected,
    }
}

#[derive(Debug)]
pub struct Writer<H, M, O>
where
    O: bincode::Options + Clone,
{
    socket: OwnedWriteHalf,
    send_buffers: VecDeque<Bytes>,
    serialization_options: O,
    _compiler_trickery: PhantomData<(H, M)>,
}

impl<H, M, O> Writer<H, M, O>
where
    H: crate::header::IsHeader,
    M: Serialize + DeserializeOwned,
    O: bincode::Options + Clone,
{
    /// Creates a new [`Writer`]
    pub fn new(socket: OwnedWriteHalf, seri_opt: O) -> Self {
        Self {
            socket,
            send_buffers: VecDeque::new(),
            serialization_options: seri_opt,
            _compiler_trickery: PhantomData,
        }
    }

    /// Queues a message to be sent
    ///
    /// # Errors
    /// if the mesage could not be serialized
    pub fn queue(
        &mut self,
        message: &crate::msg::MessageWrapper<M, H>,
    ) -> Result<(), error::SeriError> {
        let bytes = message.serialize(self.serialization_options.clone())?;
        self.send_buffers.push_back(bytes);
        Ok(())
    }

    /// Writes stored data to the socket
    ///
    /// # Errors
    /// If the socket has closed (returns Ok(0)) or if there was a error writing to the socket.
    pub async fn write(&mut self) -> Result<(), error::WriteError> {
        if self.send_buffers.is_empty() {
            Ok(())
        } else {
            // this is not undefined behavior because of the prev check to is_empty()
            let latest_buf = unsafe { self.send_buffers.get_mut(0).unwrap_unchecked() };
            match self.socket.write_buf(latest_buf).await {
                Ok(0) => {
                    if latest_buf.remaining() == 0 {
                        self.send_buffers.pop_front();
                        Ok(())
                    } else {
                        Err(error::WriteError::Disconnected)
                    }
                }
                Ok(_n) => Ok(()),
                Err(e) => Err(e.into()),
            }
        }
    }

    pub fn as_socket(&self) -> &OwnedWriteHalf {
        &self.socket
    }

    pub fn as_socket_mut(&mut self) -> &mut OwnedWriteHalf {
        &mut self.socket
    }

    pub fn into_socket(self) -> OwnedWriteHalf {
        self.socket
    }
}
