mod connect;
mod parse;

use crate::connection::LogSettings;

/// Options and flags which can be used to configure a MSSQL connection.
///
/// A value of `MssqlConnectOptions` can be parsed from a connection URL,
/// as described below.
///
/// The generic format of the connection URL:
///
/// ```text
/// mssql://[user[:password]@]host[:port][/database][?properties]
/// ```
///
/// ## Properties
///
/// |Parameter|Default|Description|
/// |---------|-------|-----------|
/// | `encrypt` | `false` | Whether to use TLS encryption. |
/// | `trust_server_certificate` | `false` | Whether to trust the server certificate without validation. |
/// | `statement-cache-capacity` | `100` | The maximum number of prepared statements stored in the cache. |
/// | `app_name` | `sqlx` | The application name sent to the server. |
/// | `instance` | `None` | The SQL Server instance name. |
///
/// # Example
///
/// ```rust,no_run
/// # async fn example() -> sqlx::Result<()> {
/// use sqlx::{Connection, ConnectOptions};
/// use sqlx::mssql::{MssqlConnectOptions, MssqlConnection};
///
/// // URL connection string
/// let conn = MssqlConnection::connect("mssql://sa:password@localhost/master").await?;
///
/// // Manually-constructed options
/// let conn = MssqlConnectOptions::new()
///     .host("localhost")
///     .username("sa")
///     .password("password")
///     .database("master")
///     .connect().await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct MssqlConnectOptions {
    pub(crate) host: String,
    pub(crate) port: u16,
    pub(crate) username: String,
    pub(crate) password: Option<String>,
    pub(crate) database: Option<String>,
    pub(crate) instance: Option<String>,
    pub(crate) encrypt: bool,
    pub(crate) trust_server_certificate: bool,
    pub(crate) statement_cache_capacity: usize,
    pub(crate) app_name: String,
    pub(crate) log_settings: LogSettings,
}

impl Default for MssqlConnectOptions {
    fn default() -> Self {
        Self::new()
    }
}

impl MssqlConnectOptions {
    /// Creates a new, default set of options ready for configuration.
    pub fn new() -> Self {
        Self {
            port: 1433,
            host: String::from("localhost"),
            username: String::from("sa"),
            password: None,
            database: None,
            instance: None,
            encrypt: false,
            trust_server_certificate: false,
            statement_cache_capacity: 100,
            app_name: String::from("sqlx"),
            log_settings: Default::default(),
        }
    }

    /// Sets the name of the host to connect to.
    pub fn host(mut self, host: &str) -> Self {
        host.clone_into(&mut self.host);
        self
    }

    /// Sets the port to connect to at the server host.
    ///
    /// The default port for MSSQL is `1433`.
    pub fn port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    /// Sets the username to connect as.
    pub fn username(mut self, username: &str) -> Self {
        username.clone_into(&mut self.username);
        self
    }

    /// Sets the password to connect with.
    pub fn password(mut self, password: &str) -> Self {
        self.password = Some(password.to_owned());
        self
    }

    /// Sets the database name.
    pub fn database(mut self, database: &str) -> Self {
        self.database = Some(database.to_owned());
        self
    }

    /// Sets the SQL Server instance name.
    pub fn instance(mut self, instance: &str) -> Self {
        self.instance = Some(instance.to_owned());
        self
    }

    /// Sets whether to use TLS encryption.
    pub fn encrypt(mut self, encrypt: bool) -> Self {
        self.encrypt = encrypt;
        self
    }

    /// Sets whether to trust the server certificate without validation.
    pub fn trust_server_certificate(mut self, trust: bool) -> Self {
        self.trust_server_certificate = trust;
        self
    }

    /// Sets the capacity of the connection's statement cache.
    pub fn statement_cache_capacity(mut self, capacity: usize) -> Self {
        self.statement_cache_capacity = capacity;
        self
    }

    /// Sets the application name sent to the server.
    pub fn app_name(mut self, app_name: &str) -> Self {
        app_name.clone_into(&mut self.app_name);
        self
    }

    /// Get the current host.
    pub fn get_host(&self) -> &str {
        &self.host
    }

    /// Get the server's port.
    pub fn get_port(&self) -> u16 {
        self.port
    }

    /// Get the current username.
    pub fn get_username(&self) -> &str {
        &self.username
    }

    /// Get the current database name.
    pub fn get_database(&self) -> Option<&str> {
        self.database.as_deref()
    }

    /// Build a `tiberius::Config` from these options.
    pub(crate) fn to_tiberius_config(&self) -> tiberius::Config {
        let mut config = tiberius::Config::new();

        config.host(&self.host);
        config.port(self.port);
        config.application_name(&self.app_name);

        if let Some(database) = &self.database {
            config.database(database);
        }

        if let Some(instance) = &self.instance {
            config.instance_name(instance);
        }

        config.authentication(tiberius::AuthMethod::sql_server(
            &self.username,
            self.password.as_deref().unwrap_or(""),
        ));

        if self.trust_server_certificate {
            config.trust_cert();
        }

        if self.encrypt {
            config.encryption(tiberius::EncryptionLevel::Required);
        } else {
            config.encryption(tiberius::EncryptionLevel::NotSupported);
        }

        config
    }
}
