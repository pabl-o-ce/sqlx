use std::fmt;

/// SQL Server transaction isolation levels.
///
/// SQL Server supports five isolation levels. The `SET TRANSACTION ISOLATION LEVEL`
/// statement must be issued **before** `BEGIN TRANSACTION`, unlike PostgreSQL which
/// accepts it inside the `BEGIN` block.
///
/// See [SQL Server documentation](https://learn.microsoft.com/en-us/sql/t-sql/statements/set-transaction-isolation-level-transact-sql)
/// for details on each level.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum MssqlIsolationLevel {
    /// Allows dirty reads. Statements can read rows modified by other
    /// transactions that have not yet been committed.
    ReadUncommitted,

    /// The default isolation level. Statements cannot read data modified
    /// by other transactions that have not been committed.
    #[default]
    ReadCommitted,

    /// Statements cannot read data modified by other transactions that
    /// have not been committed, and no other transactions can modify
    /// data read by the current transaction until it completes.
    RepeatableRead,

    /// Uses row versioning to provide transaction-level read consistency.
    /// Requires the `ALLOW_SNAPSHOT_ISOLATION` database option to be `ON`.
    Snapshot,

    /// Statements cannot read data modified by other transactions that
    /// have not been committed. No other transactions can modify data
    /// read by the current transaction, and no other transactions can
    /// insert new rows matching the current transaction's search conditions.
    Serializable,
}

impl MssqlIsolationLevel {
    /// Returns the SQL Server syntax for this isolation level.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ReadUncommitted => "READ UNCOMMITTED",
            Self::ReadCommitted => "READ COMMITTED",
            Self::RepeatableRead => "REPEATABLE READ",
            Self::Snapshot => "SNAPSHOT",
            Self::Serializable => "SERIALIZABLE",
        }
    }
}

impl fmt::Display for MssqlIsolationLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}
