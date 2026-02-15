use crate::value::{MssqlValue, MssqlValueRef};
use crate::{
    MssqlArguments, MssqlColumn, MssqlConnection, MssqlQueryResult, MssqlRow, MssqlStatement,
    MssqlTransactionManager, MssqlTypeInfo,
};
pub(crate) use sqlx_core::database::{Database, HasStatementCache};

/// MSSQL (SQL Server) database driver.
#[derive(Debug)]
pub struct Mssql;

impl Database for Mssql {
    type Connection = MssqlConnection;

    type TransactionManager = MssqlTransactionManager;

    type Row = MssqlRow;

    type QueryResult = MssqlQueryResult;

    type Column = MssqlColumn;

    type TypeInfo = MssqlTypeInfo;

    type Value = MssqlValue;
    type ValueRef<'r> = MssqlValueRef<'r>;

    type Arguments = MssqlArguments;
    type ArgumentBuffer = Vec<MssqlArgumentValue>;

    type Statement = MssqlStatement;

    const NAME: &'static str = "MSSQL";

    const URL_SCHEMES: &'static [&'static str] = &["mssql", "sqlserver"];
}

impl HasStatementCache for Mssql {}

/// A single argument value for MSSQL queries.
///
/// Unlike MySQL/Postgres which use a byte buffer, MSSQL arguments are stored
/// as typed enum variants because tiberius requires typed `bind()` calls.
#[derive(Debug, Clone)]
pub enum MssqlArgumentValue {
    Null,
    Bool(bool),
    U8(u8),
    I16(i16),
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
    String(String),
    Binary(Vec<u8>),
    #[cfg(feature = "chrono")]
    NaiveDateTime(chrono::NaiveDateTime),
    #[cfg(feature = "chrono")]
    NaiveDate(chrono::NaiveDate),
    #[cfg(feature = "chrono")]
    NaiveTime(chrono::NaiveTime),
    #[cfg(feature = "uuid")]
    Uuid(uuid::Uuid),
    #[cfg(feature = "rust_decimal")]
    Decimal(rust_decimal::Decimal),
}
