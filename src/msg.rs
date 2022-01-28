use std::marker::PhantomData;

use bytes::{BufMut, Bytes};
use serde::{Deserialize, Serialize};

use crate::header::IsHeader;

pub struct MessageWrapper<'de, M, H>
where
    M: ?Sized + Serialize + Deserialize<'de>,
{
    inner: M,
    _deserialize_lifetime: PhantomData<&'de M>,
    _header_type: PhantomData<H>,
}

impl<'de, M, H> MessageWrapper<'de, M, H>
where
    M: ?Sized + Serialize + Deserialize<'de>,
    H: IsHeader,
{
    /// Creates a new message wrapper, around a message
    pub fn new(msg: M) -> Self {
        Self {
            inner: msg,
            _deserialize_lifetime: PhantomData,
            _header_type: PhantomData,
        }
    }

    /// Create a header of the contained message
    pub fn header(&self, options: impl bincode::Options) -> Result<impl IsHeader, bincode::Error> {
        Ok(H::new(options.serialized_size(&self.inner)?))
    }

    /// Serialize the contained message, but only that, do not include the header
    pub fn serialize_self(
        &self,
        options: impl bincode::Options,
    ) -> Result<Vec<u8>, bincode::Error> {
        Ok(options.serialize(&self.inner)?)
    }

    /// Serialize and combine the header and message
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

    pub fn from_bytes<'nde, NH, NM>(
        data: &'nde Bytes,
        options: impl bincode::Options,
    ) -> Result<MessageWrapper<'nde, NM, NH>, bincode::Error>
    where
        NH: IsHeader,
        NM: ?Sized + Serialize + Deserialize<'nde>
    {
        Ok(MessageWrapper::new(options.deserialize(data)?))
    }
}

