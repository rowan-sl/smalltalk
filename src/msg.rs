use std::{marker::PhantomData, fmt::Debug};

use bytes::{BufMut, Bytes};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::header::IsHeader;

pub struct MessageWrapper<M, H>
where
    M: Serialize,
{
    inner: M,
    _header_type: PhantomData<H>,
}

impl<M, H> MessageWrapper<M, H>
where
    M: Serialize,
    H: IsHeader,
{
    /// Creates a new message wrapper, around a message
    pub fn new(msg: M) -> Self {
        Self {
            inner: msg,
            _header_type: PhantomData,
        }
    }

    /// Create a header of the contained message
    /// 
    /// # Errors
    /// if the wrappers message could not be serialized
    pub fn header(&self, options: impl bincode::Options) -> Result<impl IsHeader, bincode::Error> {
        Ok(H::new(options.serialized_size(&self.inner)?))
    }

    /// Serialize the contained message, but only that, do not include the header
    #[allow(clippy::missing_errors_doc)]
    pub fn serialize_self(
        &self,
        options: impl bincode::Options,
    ) -> Result<Vec<u8>, bincode::Error> {
        options.serialize(&self.inner)
    }

    /// Serialize and combine the header and message
    #[allow(clippy::missing_errors_doc)]
    pub fn serialize(
        &self,
        options: impl bincode::Options + Clone,
    ) -> Result<Bytes, bincode::Error> {
        let mut header = self.header(options.clone())?.as_bytes_mut();
        let serialized_self = self.serialize_self(options)?;
        header.reserve(serialized_self.len());
        header.put_slice(&serialized_self);
        Ok(header.freeze())
    }

    /// Consumes self, producing the contained message
    pub fn into_message(self) -> M {
        self.inner
    }

    pub fn message(&self) -> &M {
        &self.inner
    }

    /// Mutable reference to the contained message.
    /// ## WARNING!
    /// if you serialized or retreived a header before doing this, it is now incorrect!
    pub fn message_mut(&mut self) -> &mut M {
        &mut self.inner
    }

    /// Attempts to deserialize a message from the provided data
    /// 
    /// # Errors
    /// if the message could not be deserialized
    pub fn from_bytes<NH, NM>(
        data: &Bytes,
        options: impl bincode::Options,
    ) -> Result<MessageWrapper<NM, NH>, bincode::Error>
    where
        NH: IsHeader,
        NM: Serialize + DeserializeOwned,
    {
        Ok(MessageWrapper::new(options.deserialize(data)?))
    }

    /// Attempts to deserialize a message from the provided data
    /// 
    /// # Errors
    /// if the message could not be deserialized
    pub fn from_slice<'nde, NH, NM>(
        data: &'nde &[u8],
        options: impl bincode::Options,
    ) -> Result<MessageWrapper<NM, NH>, bincode::Error>
    where
        NH: IsHeader,
        NM: Serialize + Deserialize<'nde>,
    {
        Ok(MessageWrapper::new(options.deserialize(data)?))
    }
}

impl<M, H> Debug for MessageWrapper<M, H>
where
    M: Serialize + Debug,
    H: IsHeader + Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MessageWrapper")
            .field("message", &self.inner)
            .finish()
    }
}