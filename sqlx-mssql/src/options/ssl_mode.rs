/// The SSL mode to use when connecting to MSSQL.
///
/// Maps to the tiberius `EncryptionLevel` variants.
///
/// Default is `Disabled` because `tiberius` is pulled in without its `rustls`
/// or `native-tls` feature, so it cannot perform a TLS handshake. Selecting
/// `LoginOnly`, `Preferred`, or `Required` without enabling tiberius's TLS
/// support will fail at connection time with an EOF on the TLS handshake.
#[derive(Debug, Clone, Copy, Default)]
pub enum MssqlSslMode {
    /// No encryption at all (`EncryptionLevel::NotSupported`).
    #[default]
    Disabled,

    /// Only encrypt the login packet (`EncryptionLevel::Off`).
    LoginOnly,

    /// Encrypt if the server supports it (`EncryptionLevel::On`).
    Preferred,

    /// Always encrypt; fail if the server doesn't support it (`EncryptionLevel::Required`).
    Required,
}
