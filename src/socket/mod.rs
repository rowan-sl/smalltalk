pub mod read;
pub mod write;

use serde::{de::DeserializeOwned, Serialize};
use tokio::net::TcpStream;

pub use read::Reader;
pub use write::Writer;

/// Splits a `TcpStream` into a `Reader` and `Writer`
pub fn split_stream<H, M, O>(stream: TcpStream, seri_opt: O) -> (Reader<H, M, O>, Writer<H, M, O>)
where
    H: crate::header::IsHeader + Clone,
    M: Serialize + DeserializeOwned,
    O: bincode::Options + Clone,
{
    let (read_half, write_half) = stream.into_split();
    (
        Reader::new(read_half, seri_opt.clone()),
        Writer::new(write_half, seri_opt),
    )
}

/// Attempts to join a `Reader` and `Writer` into a `TcpStream`
///
/// # Errors
/// if the halfs did not originate from the same `TcpStream`
pub fn join_stream<H, M, O>(
    read_half: Reader<H, M, O>,
    write_half: Writer<H, M, O>,
) -> Result<TcpStream, tokio::net::tcp::ReuniteError>
where
    H: crate::header::IsHeader + Clone,
    M: Serialize + DeserializeOwned,
    O: bincode::Options + Clone,
{
    read_half.into_socket().reunite(write_half.into_socket())
}
