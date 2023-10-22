pub mod cmd;
pub use cmd::Command;

mod connection;
pub use connection::Connection;

pub mod frame;
pub use frame::Frame;

mod db;
use db::Db;
use db::DbDropGuard;

mod parse;
use parse::Parse;
use parse::ParseError;

mod shutdown;
use shutdown::Shutdown;

/// Default port that a redis server listens on. 
///
/// Used if no port is specified. 
pub const DEFAULT_PORT: u16 = 6379;

pub type Error = Box<dyn std::error::Error + Send + Sync>;

/// A specialized `Result` type for mini-redis operations
///
/// This is defined as a convenience
pub type Result<T> = std::result::Result<T, Error>;
