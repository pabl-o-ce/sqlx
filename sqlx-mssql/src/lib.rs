//! **MSSQL** (SQL Server) database driver.
#![deny(clippy::cast_possible_truncation)]
#![deny(clippy::cast_possible_wrap)]
#![deny(clippy::cast_sign_loss)]

#[macro_use]
extern crate sqlx_core;

use crate::executor::Executor;

pub(crate) use sqlx_core::driver_prelude::*;

pub mod advisory_lock;
mod bulk_insert;
mod isolation_level;

#[cfg(feature = "any")]
pub mod any;

mod arguments;
mod column;
mod connection;
mod database;
mod error;
mod io;
mod options;
mod query_result;
mod row;
mod statement;
mod transaction;
mod type_checking;
mod type_info;
pub mod types;
mod value;

#[cfg(feature = "migrate")]
mod migrate;

#[cfg(feature = "migrate")]
mod testing;

pub use advisory_lock::{MssqlAdvisoryLock, MssqlAdvisoryLockMode};
pub use arguments::MssqlArguments;
pub use bulk_insert::MssqlBulkInsert;
pub use column::MssqlColumn;
pub use connection::MssqlConnection;
pub use database::Mssql;
pub use error::MssqlDatabaseError;
pub use isolation_level::MssqlIsolationLevel;
pub use options::ssl_mode::MssqlSslMode;
pub use options::MssqlConnectOptions;
pub use query_result::MssqlQueryResult;
pub use row::MssqlRow;
pub use statement::MssqlStatement;
pub use transaction::MssqlTransactionManager;
pub use type_info::MssqlTypeInfo;
pub use value::{MssqlValue, MssqlValueRef};

// Re-export tiberius types needed for bulk insert row construction.
pub use tiberius::{IntoRow, IntoSql, TokenRow};

/// An alias for [`Pool`][crate::pool::Pool], specialized for MSSQL.
pub type MssqlPool = crate::pool::Pool<Mssql>;

/// An alias for [`PoolOptions`][crate::pool::PoolOptions], specialized for MSSQL.
pub type MssqlPoolOptions = crate::pool::PoolOptions<Mssql>;

/// An alias for [`Executor<'_, Database = Mssql>`][Executor].
pub trait MssqlExecutor<'c>: Executor<'c, Database = Mssql> {}
impl<'c, T: Executor<'c, Database = Mssql>> MssqlExecutor<'c> for T {}

/// An alias for [`Transaction`][crate::transaction::Transaction], specialized for MSSQL.
pub type MssqlTransaction<'c> = crate::transaction::Transaction<'c, Mssql>;

// NOTE: required due to the lack of lazy normalization
impl_into_arguments_for_arguments!(MssqlArguments);
impl_acquire!(Mssql, MssqlConnection);
impl_column_index_for_row!(MssqlRow);
impl_column_index_for_statement!(MssqlStatement);

// required because some databases have a different handling of NULL
impl_encode_for_option!(Mssql);
