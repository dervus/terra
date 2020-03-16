use crate::errors::TerraResult;

pub type RedisPool = mobc::Pool<mobc_redis::RedisConnectionManager>;
pub type RedisConn = mobc::Connection<mobc_redis::RedisConnectionManager>;
pub type RedisError = mobc_redis::redis::RedisError;
pub type MysqlPool = mysql_async::Pool;
pub type MysqlConn = mysql_async::Conn;
pub type MysqlError = mysql_async::error::Error;
pub type MysqlResult<T> = TerraResult<(MysqlConn, T)>;

mod account;
mod session;
mod character;

pub use account::*;
pub use session::*;
pub use character::*;
