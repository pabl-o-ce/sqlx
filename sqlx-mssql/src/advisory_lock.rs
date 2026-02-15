use crate::error::Error;
use crate::query_scalar::query_scalar;
use crate::MssqlConnection;

/// The lock mode for a MSSQL advisory lock.
///
/// Maps to the `@LockMode` parameter of `sp_getapplock`.
#[derive(Debug, Clone, Copy, Default)]
pub enum MssqlAdvisoryLockMode {
    /// A shared lock, compatible with other `Shared` and `Update` locks.
    Shared,

    /// An update lock, compatible with `Shared` but not with other `Update` or `Exclusive`.
    Update,

    /// An exclusive lock, incompatible with all other lock modes.
    #[default]
    Exclusive,
}

impl MssqlAdvisoryLockMode {
    fn as_str(&self) -> &'static str {
        match self {
            MssqlAdvisoryLockMode::Shared => "Shared",
            MssqlAdvisoryLockMode::Update => "Update",
            MssqlAdvisoryLockMode::Exclusive => "Exclusive",
        }
    }
}

/// A session-scoped advisory lock backed by SQL Server's `sp_getapplock` /
/// `sp_releaseapplock`.
///
/// Advisory locks are cooperative: they don't block access to any database
/// object; instead, all participants must explicitly acquire the same named
/// lock. The lock is scoped to the database session (connection).
///
/// Unlike the Postgres advisory-lock API, there is **no RAII drop guard**.
/// You must call [`release`][Self::release] explicitly when you are done with
/// the lock.
///
/// # Resource Name
///
/// SQL Server limits resource names to 255 characters. The name is passed as a
/// query parameter, so SQL injection is not possible.
///
/// # Example
///
/// ```rust,no_run
/// # async fn example(conn: &mut sqlx::mssql::MssqlConnection) -> sqlx::Result<()> {
/// use sqlx::mssql::MssqlAdvisoryLock;
///
/// let lock = MssqlAdvisoryLock::new("my_app_lock");
/// lock.acquire(conn).await?;
///
/// // ... do work under the lock ...
///
/// lock.release(conn).await?;
/// # Ok(())
/// # }
/// ```
pub struct MssqlAdvisoryLock {
    resource: String,
    mode: MssqlAdvisoryLockMode,
}

impl MssqlAdvisoryLock {
    /// Create a new advisory lock with the given resource name and the default
    /// [`Exclusive`][MssqlAdvisoryLockMode::Exclusive] mode.
    pub fn new(resource: impl Into<String>) -> Self {
        Self {
            resource: resource.into(),
            mode: MssqlAdvisoryLockMode::default(),
        }
    }

    /// Create a new advisory lock with the given resource name and lock mode.
    pub fn with_mode(resource: impl Into<String>, mode: MssqlAdvisoryLockMode) -> Self {
        Self {
            resource: resource.into(),
            mode,
        }
    }

    /// Returns the resource name of this lock.
    pub fn resource(&self) -> &str {
        &self.resource
    }

    /// Returns the lock mode.
    pub fn mode(&self) -> &MssqlAdvisoryLockMode {
        &self.mode
    }

    /// Acquire the lock, waiting indefinitely until it is available.
    ///
    /// # Errors
    ///
    /// Returns an error if `sp_getapplock` returns a negative status code
    /// (e.g. lock request was cancelled or a deadlock was detected).
    pub async fn acquire(&self, conn: &mut MssqlConnection) -> Result<(), Error> {
        let mode = self.mode.as_str();
        let sql = format!(
            "DECLARE @r INT; \
             EXEC @r = sp_getapplock @Resource = @p1, @LockMode = '{mode}', \
             @LockOwner = 'Session', @LockTimeout = -1; \
             SELECT @r;"
        );

        let status: i32 = query_scalar(sqlx_core::sql_str::AssertSqlSafe(sql))
            .bind(&self.resource)
            .fetch_one(&mut *conn)
            .await?;

        if status < 0 {
            return Err(Error::Protocol(format!(
                "sp_getapplock failed for resource '{}': status {status}{}",
                self.resource,
                applock_error_message(status),
            )));
        }

        Ok(())
    }

    /// Try to acquire the lock without waiting.
    ///
    /// Returns `Ok(true)` if the lock was acquired, `Ok(false)` if it was not
    /// available (timeout).
    pub async fn try_acquire(&self, conn: &mut MssqlConnection) -> Result<bool, Error> {
        let mode = self.mode.as_str();
        let sql = format!(
            "DECLARE @r INT; \
             EXEC @r = sp_getapplock @Resource = @p1, @LockMode = '{mode}', \
             @LockOwner = 'Session', @LockTimeout = 0; \
             SELECT @r;"
        );

        let status: i32 = query_scalar(sqlx_core::sql_str::AssertSqlSafe(sql))
            .bind(&self.resource)
            .fetch_one(&mut *conn)
            .await?;

        if status >= 0 {
            // 0 = granted synchronously, 1 = granted after wait
            Ok(true)
        } else if status == -1 {
            // -1 = timed out
            Ok(false)
        } else {
            Err(Error::Protocol(format!(
                "sp_getapplock failed for resource '{}': status {status}{}",
                self.resource,
                applock_error_message(status),
            )))
        }
    }

    /// Release the lock.
    ///
    /// Returns `Ok(true)` if the lock was successfully released, `Ok(false)`
    /// if the lock was not held by this session.
    pub async fn release(&self, conn: &mut MssqlConnection) -> Result<bool, Error> {
        let sql = "DECLARE @r INT; \
                   EXEC @r = sp_releaseapplock @Resource = @p1, @LockOwner = 'Session'; \
                   SELECT @r;";

        let status: i32 = query_scalar(sql)
            .bind(&self.resource)
            .fetch_one(&mut *conn)
            .await?;

        match status {
            0 => Ok(true),
            -999 => Ok(false),
            _ => Err(Error::Protocol(format!(
                "sp_releaseapplock failed for resource '{}': status {status}",
                self.resource,
            ))),
        }
    }
}

fn applock_error_message(status: i32) -> &'static str {
    match status {
        -1 => " (timed out)",
        -2 => " (lock request cancelled)",
        -3 => " (deadlock victim)",
        -999 => " (parameter validation or other call error)",
        _ => "",
    }
}
