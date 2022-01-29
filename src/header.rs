use std::fmt::{Debug, Display};

use bytes::{Bytes, BytesMut};

/// Trait for methods that should be found on header implementations
pub trait IsHeader {
    type Error: Debug + Display;

    /// Create a new header
    #[must_use]
    fn new(msg_len: u64) -> Self
    where
        Self: Sized;

    /// Create a new header with a message size of zero
    #[must_use]
    fn blank() -> Self
    where
        Self: Sized,
    {
        Self::new(0)
    }

    /// Get the size of the message contained within
    #[must_use]
    fn size(&self) -> u64;

    /// Get the header, represented as bytes
    #[must_use]
    fn as_bytes(&self) -> Bytes;

    /// Get the header, represented as bytes
    #[must_use]
    fn as_bytes_mut(&self) -> BytesMut;

    /// Create a new header, from some bytes.
    /// This should do all necessary validation checks.
    ///
    /// # Errors
    /// if the header contained in `bytes` was invalid
    fn from_bytes(bytes: Bytes) -> Result<Self, Self::Error>
    where
        Self: Sized;

    /// size of the header. this should never change and is used to read the appropreate number of bytes when deserializing
    #[must_use]
    fn header_size() -> usize;
}
