pub mod client;
pub mod header;
pub mod msg;
pub mod server;
pub mod socket;

pub use header::IsHeader;
pub use msg::MessageWrapper;
pub use socket::{Reader, Writer};
pub use server::Server;
pub use client::Client;

/// Trait imports for smalltalk.
///
/// This is so you can use methods found on these traits,
/// if you want to use the traits themselves import them seperatly.
///
/// All traits are renamed `_smalltalk_<trait name>` to avoid confusion and
/// not knowing where imported items came from.
pub mod prelude {
    pub use crate::header::IsHeader as _smalltalk_IsHeader;
}
