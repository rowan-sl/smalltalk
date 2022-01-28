pub mod msg;
pub mod old;
pub mod header;

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

fn main() {
    println!("Hello, world!");
}
