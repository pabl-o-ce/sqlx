use std::str::FromStr;

use percent_encoding::percent_decode_str;
use sqlx_core::Url;

use crate::error::Error;

use super::MssqlConnectOptions;

impl MssqlConnectOptions {
    pub(crate) fn parse_from_url(url: &Url) -> Result<Self, Error> {
        let mut options = Self::new();

        if let Some(host) = url.host_str() {
            options = options.host(host);
        }

        if let Some(port) = url.port() {
            options = options.port(port);
        }

        let username = url.username();
        if !username.is_empty() {
            options = options.username(
                &percent_decode_str(username)
                    .decode_utf8()
                    .map_err(Error::config)?,
            );
        }

        if let Some(password) = url.password() {
            options = options.password(
                &percent_decode_str(password)
                    .decode_utf8()
                    .map_err(Error::config)?,
            );
        }

        let path = url.path().trim_start_matches('/');
        if !path.is_empty() {
            options = options.database(
                &percent_decode_str(path)
                    .decode_utf8()
                    .map_err(Error::config)?,
            );
        }

        for (key, value) in url.query_pairs().into_iter() {
            match &*key {
                "encrypt" => {
                    options = options
                        .encrypt(value.parse().map_err(Error::config)?);
                }

                "trust_server_certificate" | "trustServerCertificate" => {
                    options = options
                        .trust_server_certificate(value.parse().map_err(Error::config)?);
                }

                "instance" => {
                    options = options.instance(&value);
                }

                "app_name" | "application-name" => {
                    options = options.app_name(&value);
                }

                "statement-cache-capacity" => {
                    options = options
                        .statement_cache_capacity(value.parse().map_err(Error::config)?);
                }

                _ => {}
            }
        }

        Ok(options)
    }

    pub(crate) fn build_url(&self) -> Url {
        let mut url = Url::parse(&format!(
            "mssql://{}@{}:{}",
            self.username, self.host, self.port
        ))
        .expect("BUG: generated un-parseable URL");

        if let Some(password) = &self.password {
            let _ = url.set_password(Some(password));
        }

        if let Some(database) = &self.database {
            url.set_path(database);
        }

        url
    }
}

impl FromStr for MssqlConnectOptions {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Error> {
        let url: Url = s.parse().map_err(Error::config)?;
        Self::parse_from_url(&url)
    }
}

#[test]
fn it_parses_basic_mssql_url() {
    let url = "mssql://sa:password@localhost:1433/master";
    let opts = MssqlConnectOptions::from_str(url).unwrap();

    assert_eq!(opts.host, "localhost");
    assert_eq!(opts.port, 1433);
    assert_eq!(opts.username, "sa");
    assert_eq!(opts.password, Some("password".into()));
    assert_eq!(opts.database, Some("master".into()));
}

#[test]
fn it_parses_url_with_instance() {
    let url = "mssql://sa:password@localhost/master?instance=SQLEXPRESS";
    let opts = MssqlConnectOptions::from_str(url).unwrap();

    assert_eq!(opts.instance, Some("SQLEXPRESS".into()));
}
