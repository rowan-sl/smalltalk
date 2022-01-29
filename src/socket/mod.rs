pub mod read;
pub mod write;

use serde::{de::DeserializeOwned, Serialize};
use tokio::net::TcpStream;

pub use read::SocketReader;
pub use write::SocketWriter;

/// Splits a `TcpStream` into a `SocketReader` and `SocketWriter`
pub fn split_stream<H, M, O>(
    stream: TcpStream,
    seri_opt: O,
) -> (SocketReader<H, M, O>, SocketWriter<H, M, O>)
where
    H: crate::header::IsHeader + Clone,
    M: Serialize + DeserializeOwned,
    O: bincode::Options + Clone,
{
    let (read_half, write_half) = stream.into_split();
    (
        SocketReader::new(read_half, seri_opt.clone()),
        SocketWriter::new(write_half, seri_opt),
    )
}

/// attempts to join a SocketReader and SocketWriter into a TcpStream,
/// failing if they were not from the same stream originaly
pub fn join_stream<H, M, O>(
    read_half: SocketReader<H, M, O>,
    write_half: SocketWriter<H, M, O>,
) -> Result<TcpStream, tokio::net::tcp::ReuniteError>
where
    H: crate::header::IsHeader + Clone,
    M: Serialize + DeserializeOwned,
    O: bincode::Options + Clone,
{
    read_half.into_socket().reunite(write_half.into_socket())
}