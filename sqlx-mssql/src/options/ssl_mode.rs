/// The SSL mode to use when connecting to MSSQL.
///
/// Maps to the tiberius `EncryptionLevel` variants.
#[derive(Debug, Clone, Copy, Default)]
pub enum MssqlSslMode {
    /// No encryption at all (`EncryptionLevel::NotSupported`).
    Disabled,

    /// Only encrypt the login packet (`EncryptionLevel::Off`).
    LoginOnly,

    /// Encrypt if the server supports it (`EncryptionLevel::On`).
    #[default]
    Preferred,

    /// Always encrypt; fail if the server doesn't support it (`EncryptionLevel::Required`).
    Required,
}
