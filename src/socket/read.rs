use bytes::BytesMut;
use serde::{de::DeserializeOwned, Serialize};
use tokio::{io::AsyncReadExt, net::tcp::OwnedReadHalf};

enum SocketReaderState<H>
where
    H: crate::header::IsHeader,
{
    Ready,
    ReadingHeader,
    ProcessHeader,
    ReadingMessage { header: H },
    ProcessMessage { header: H },
}

impl<H> Default for SocketReaderState<H>
where
    H: crate::header::IsHeader,
{
    fn default() -> Self {
        Self::Ready
    }
}

pub mod error {
    #[derive(thiserror::Error, Debug)]
    pub enum SocketReaderUpdateError<H>
    where
        H: crate::header::IsHeader,
    {
        #[error("Failed to parse header {0}")]
        HeaderParser(H::Error),
        #[error("Failed to deserialize message {0}")]
        MessageDeseri(#[from] bincode::Error),
    }
}

pub struct SocketReader<H, M, O>
where
    H: crate::header::IsHeader,
    M: Serialize + DeserializeOwned,
    O: bincode::Options,
{
    socket: OwnedReadHalf,
    databuffer: BytesMut,
    state: SocketReaderState<H>,
    ready_messages: Vec<crate::msg::MessageWrapper<M, H>>,
    serialization_settings: O,
    /// convenience for `H::header_size()`
    header_size: usize,
}

impl<H, M, O> SocketReader<H, M, O>
where
    H: crate::header::IsHeader + Clone,
    M: Serialize + DeserializeOwned,
    O: bincode::Options + Clone,
{
    pub fn new(socket: OwnedReadHalf, seri_settings: O) -> Self {
        Self {
            socket,
            databuffer: BytesMut::new(),
            state: Default::default(),
            ready_messages: vec![],
            serialization_settings: seri_settings,
            header_size: H::header_size(),
        }
    }

    /// attempts to read and store data. this does NOT attempt to read more than once,
    /// and does NOT process the data.
    ///
    /// ## Cancelation Saftey
    /// this method IS cancelation safe. no data will be lost if it is canceled
    ///
    /// ## Errors
    /// when the underlying socket.read() returns a io error
    pub async fn read(&mut self) -> std::io::Result<()> {
        self.socket.read_buf(&mut self.databuffer).await?;

        // here, only the reading variants are used.
        // a reading variant, like ReadingHeader, should have the option to progress to the processing variant,
        // like ProcessHeader, once it receives enough data
        // processing stages are dealt with elsewhere
        match self.state {
            SocketReaderState::Ready => {
                self.state = SocketReaderState::ReadingHeader;
            }
            SocketReaderState::ReadingHeader => {
                if self.databuffer.len() >= self.header_size {
                    // we art r e a d y
                    self.state = SocketReaderState::ProcessHeader;
                }
            }
            SocketReaderState::ReadingMessage { ref header } => {
                //TODO make this not use .expect()
                if self.databuffer.len()
                    >= header
                        .size()
                        .try_into()
                        .expect("Cannot convert u64 to usize, this is probably a 32bit system")
                {
                    // dun dun done
                    self.state = SocketReaderState::ProcessMessage {
                        header: header.clone(),
                    };
                }
            }
            _ => {}
        }

        Ok(())
    }

    /// Updates the reader.
    ///
    /// does not read any bytes from the socket, but instead checks if
    /// a message is ready to be deserialized.
    ///
    /// returns if there is a new message in the result queue or not
    pub async fn update(&mut self) -> Result<bool, error::SocketReaderUpdateError<H>>
    {
        match self.state {
            SocketReaderState::ProcessHeader => {
                let header_dat = self.databuffer.split_to(self.header_size).freeze();
                match H::from_bytes(header_dat) {
                    Ok(header) => {
                        self.state = SocketReaderState::ReadingMessage { header };
                        Ok(false)
                    }
                    Err(e) => Err(error::SocketReaderUpdateError::HeaderParser(e)),
                }
            }
            SocketReaderState::ProcessMessage { ref header } => {
                //TODO remove .expect()
                let message_dat = self.databuffer.split_to(usize::try_from(header.size()).expect("Converted u64 to usize. if this fails, you are probably not on a 64 bit system and sending LARGE messages")).freeze();
                let message: crate::msg::MessageWrapper<M, H> =
                    crate::msg::MessageWrapper::<M, H>::from_owned_bytes(
                        message_dat,
                        self.serialization_settings.clone(),
                    )?;
                self.ready_messages.push(message);
                Ok(true)
            }
            _ => {
                /* ignore other things because they are related to processing messages */
                Ok(false)
            }
        }
    }

    pub fn get_ready_messages(&mut self) -> std::vec::Drain<crate::msg::MessageWrapper<M, H>> {
        self.ready_messages.drain(..)
    }

    pub fn clear_state(&mut self) {
        self.databuffer.clear();
        self.state = Default::default();
        self.ready_messages.clear();
    }

    pub fn get_socket(&self) -> &OwnedReadHalf {
        &self.socket
    }

    pub fn get_socket_mut(&mut self) -> &mut OwnedReadHalf {
        &mut self.socket
    }

    pub fn into_socket(self) -> OwnedReadHalf {
        self.socket
    }
}
