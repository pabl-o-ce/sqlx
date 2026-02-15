use std::fmt::{self, Debug, Formatter};

pub(crate) use sqlx_core::connection::*;
use sqlx_core::net::Socket;
use sqlx_core::sql_str::SqlSafeStr;

use crate::common::StatementCache;
use crate::error::Error;
use crate::executor::Executor;
use crate::io::SocketAdapter;
use crate::statement::MssqlStatementMetadata;
use crate::transaction::Transaction;
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
