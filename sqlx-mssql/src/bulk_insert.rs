use crate::error::{tiberius_err, Error};
use crate::io::SocketAdapter;
use sqlx_core::net::Socket;

/// A bulk insert operation for high-performance data loading into SQL Server.
///
/// Wraps the tiberius [`BulkLoadRequest`](tiberius::BulkLoadRequest) to provide
/// efficient bulk data insertion using the TDS `INSERT BULK` protocol.
///
/// # Example
///
/// ```rust,no_run
/// # async fn example(conn: &mut sqlx::mssql::MssqlConnection) -> sqlx::Result<()> {
/// use sqlx::mssql::IntoRow;
///
/// let mut bulk = conn.bulk_insert("#my_temp_table").await?;
/// bulk.send(("hello", 42i32).into_row()).await?;
/// bulk.send(("world", 99i32).into_row()).await?;
/// let total = bulk.finalize().await?;
/// assert_eq!(total, 2);
/// # Ok(())
/// # }
/// ```
pub struct MssqlBulkInsert<'c> {
    inner: tiberius::BulkLoadRequest<'c, SocketAdapter<Box<dyn Socket>>>,
}

impl<'c> MssqlBulkInsert<'c> {
    pub(crate) fn new(
        inner: tiberius::BulkLoadRequest<'c, SocketAdapter<Box<dyn Socket>>>,
    ) -> Self {
        Self { inner }
    }

    /// Send a single row to the bulk insert operation.
    ///
    /// The row is a [`tiberius::TokenRow`] — use [`tiberius::IntoRow::into_row()`]
    /// to convert tuples of up to 10 elements into a `TokenRow`.
    pub async fn send(&mut self, row: tiberius::TokenRow<'c>) -> Result<(), Error> {
        self.inner.send(row).await.map_err(tiberius_err)
    }

    /// Finalize the bulk insert, flushing all buffered data to the server.
    ///
    /// Returns the total number of rows inserted. This **must** be called
    /// after all rows have been sent — otherwise data will be lost.
    pub async fn finalize(self) -> Result<u64, Error> {
        let result = self.inner.finalize().await.map_err(tiberius_err)?;
        Ok(result.total())
    }
}
