use std::{collections::VecDeque, marker::PhantomData};

use bytes::{Bytes, Buf};
use serde::{de::DeserializeOwned, Serialize};
use tokio::{net::tcp::OwnedWriteHalf, io::AsyncWriteExt};

pub mod error {
    #[derive(Debug, thiserror::Error)]
    #[error("Failed to serialize message!\n{0}")]
    pub struct SeriError (#[from] bincode::Error);

    #[derive(Debug, thiserror::Error)]
    pub enum WriteError {
        #[error("Error while sending data!\n{0}")]
        IOError (#[from] std::io::Error),
        #[error("Socket Closed!")]
        Disconnected,
    }
}


pub struct SocketWriter<H, M, O>
where
    O: bincode::Options + Clone,
{
    socket: OwnedWriteHalf,
    send_buffers: VecDeque<Bytes>,
    serialization_options: O,
    _compiler_trickery: PhantomData<(H, M)>,
}

impl<H, M, O> SocketWriter<H, M, O>
where
    H: crate::header::IsHeader,
    M: Serialize + DeserializeOwned,
    O: bincode::Options + Clone,
{
    pub fn new(socket: OwnedWriteHalf, seri_opt: O) -> Self {
        Self {
            socket,
            send_buffers: VecDeque::new(),
            serialization_options: seri_opt,
            _compiler_trickery: PhantomData,
        }
    }

    pub fn queue(&mut self, message: crate::msg::MessageWrapper<M, H>) -> Result<(), error::SeriError> {
        let bytes = message.serialize(self.serialization_options.clone())?;
        self.send_buffers.push_back(bytes);    
        Ok(())
    }

    pub async fn write(&mut self) -> Result<(), error::WriteError> {
        if !self.send_buffers.is_empty() {
            // this is not undefined behavior because of the prev check to is_empty()
            let latest_buf = unsafe { self.send_buffers.get_mut(0).unwrap_unchecked() };
            match self.socket.write_buf(latest_buf).await {
                Ok(0) => {
                    if latest_buf.remaining() == 0 {
                        drop(latest_buf);
                        self.send_buffers.pop_front();
                        Ok(())
                    } else {
                        Err(error::WriteError::Disconnected)
                    }
                }
                Ok(_n) => {Ok(())}
                Err(e) => Err(e.into())
            }
        } else {
            Ok(())
        }
    }

    pub fn get_socket(&self) -> &OwnedWriteHalf {
        &self.socket
    }

    pub fn get_socket_mut(&mut self) -> &mut OwnedWriteHalf {
        &mut self.socket
    }

    pub fn into_socket(self) -> OwnedWriteHalf {
        self.socket
    }
}