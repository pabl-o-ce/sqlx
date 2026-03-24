use std::fmt::{self, Debug, Formatter};

pub(crate) use sqlx_core::connection::*;
use sqlx_core::net::Socket;
use sqlx_core::sql_str::{AssertSqlSafe, SqlSafeStr};

use crate::bulk_insert::MssqlBulkInsert;
use crate::common::StatementCache;
use crate::error::{tiberius_err, Error};
use crate::executor::Executor;
use crate::io::SocketAdapter;
use crate::isolation_level::MssqlIsolationLevel;
use crate::statement::MssqlStatementMetadata;
use crate::transaction::{resolve_pending_rollback, Transaction};
use crate::{Mssql, MssqlConnectOptions};

mod establish;
mod executor;

/// A connection to a MSSQL database.
pub struct MssqlConnection {
    pub(crate) inner: Box<MssqlConnectionInner>,
}

pub(crate) struct MssqlConnectionInner {
    pub(crate) client: tiberius::Client<SocketAdapter<Box<dyn Socket>>>,
    pub(crate) transaction_depth: usize,
    pub(crate) pending_rollback: bool,
    pub(crate) log_settings: LogSettings,
    pub(crate) cache_statement: StatementCache<MssqlStatementMetadata>,
}

impl Debug for MssqlConnection {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("MssqlConnection").finish()
    }
}

impl Connection for MssqlConnection {
    type Database = Mssql;

    type Options = MssqlConnectOptions;

    async fn close(self) -> Result<(), Error> {
        // tiberius doesn't have an explicit close; dropping the client closes the connection.
        drop(self);
        Ok(())
    }

    async fn close_hard(self) -> Result<(), Error> {
        drop(self);
        Ok(())
    }

    async fn ping(&mut self) -> Result<(), Error> {
        self.execute("SELECT 1").await?;
        Ok(())
    }

    fn flush(&mut self) -> impl std::future::Future<Output = Result<(), Error>> + Send + '_ {
        // No-op for MSSQL since tiberius handles buffering internally.
        std::future::ready(Ok(()))
    }

    fn cached_statements_size(&self) -> usize {
        self.inner.cache_statement.len()
    }

    async fn clear_cached_statements(&mut self) -> Result<(), Error> {
        self.inner.cache_statement.clear();
        Ok(())
    }

    fn should_flush(&self) -> bool {
        false
    }

    fn begin(
        &mut self,
    ) -> impl std::future::Future<Output = Result<Transaction<'_, Self::Database>, Error>> + Send + '_
    {
        Transaction::begin(self, None)
    }

    fn begin_with(
        &mut self,
        statement: impl SqlSafeStr,
    ) -> impl std::future::Future<Output = Result<Transaction<'_, Self::Database>, Error>> + Send + '_
    where
        Self: Sized,
    {
        Transaction::begin(self, Some(statement.into_sql_str()))
    }

    fn shrink_buffers(&mut self) {
        // No-op for MSSQL
    }
}

// Implement `AsMut<Self>` so that `MssqlConnection` can be wrapped in
// a `MssqlAdvisoryLockGuard`.
impl AsMut<MssqlConnection> for MssqlConnection {
    fn as_mut(&mut self) -> &mut MssqlConnection {
        self
    }
}

impl AsRef<MssqlConnection> for MssqlConnection {
    fn as_ref(&self) -> &MssqlConnection {
        self
    }
}

impl MssqlConnection {
    /// Begin a transaction with a specific isolation level.
    ///
    /// SQL Server requires `SET TRANSACTION ISOLATION LEVEL` to be issued
    /// **before** `BEGIN TRANSACTION`. This method generates:
    ///
    /// ```sql
    /// SET TRANSACTION ISOLATION LEVEL <level>; BEGIN TRANSACTION
    /// ```
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # async fn example(conn: &mut sqlx::mssql::MssqlConnection) -> sqlx::Result<()> {
    /// use sqlx::mssql::MssqlIsolationLevel;
    ///
    /// let mut tx = conn.begin_with_isolation(MssqlIsolationLevel::Snapshot).await?;
    /// // ... use tx ...
    /// tx.commit().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn begin_with_isolation(
        &mut self,
        level: MssqlIsolationLevel,
    ) -> impl std::future::Future<Output = Result<Transaction<'_, Mssql>, Error>> + Send + '_ {
        let sql = AssertSqlSafe(format!(
            "SET TRANSACTION ISOLATION LEVEL {level}; BEGIN TRANSACTION"
        ));
        Transaction::begin(self, Some(sql.into_sql_str()))
    }

    /// Start a bulk insert operation for high-performance data loading.
    ///
    /// The table must already exist. Tiberius executes `SELECT TOP 0 * FROM <table>`
    /// to discover column metadata, then uses the TDS `INSERT BULK` protocol.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # async fn example(conn: &mut sqlx::mssql::MssqlConnection) -> sqlx::Result<()> {
    /// use sqlx::mssql::IntoRow;
    ///
    /// let mut bulk = conn.bulk_insert("#temp").await?;
    /// bulk.send(("hello", 42i32).into_row()).await?;
    /// let total = bulk.finalize().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn bulk_insert<'c>(
        &'c mut self,
        table: &'c str,
    ) -> Result<MssqlBulkInsert<'c>, Error> {
        resolve_pending_rollback(self).await?;
        let req = self
            .inner
            .client
            .bulk_insert(table)
            .await
            .map_err(tiberius_err)?;
        Ok(MssqlBulkInsert::new(req))
    }
}
