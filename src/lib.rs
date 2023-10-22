mod connection;

pub mod frame;
pub use frame::Frame;

mod db;
use db::Db;
use db::DbDropGuard;

pub type Error = Box<dyn std::error::Error + Send + Sync>;

/// A specialized `Result` type for mini-redis operations
///
/// This is defined as a convenience
pub type Result<T> = std::result::Result<T, Error>;
