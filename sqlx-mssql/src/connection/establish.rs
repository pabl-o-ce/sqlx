use crate::common::StatementCache;
use crate::connection::MssqlConnectionInner;
use crate::error::{tiberius_err, Error};
use crate::io::SocketAdapter;
use crate::{MssqlConnectOptions, MssqlConnection};
use sqlx_core::net::{Socket, WithSocket};

impl MssqlConnection {
    pub(crate) async fn establish(options: &MssqlConnectOptions) -> Result<Self, Error> {
        let config = options.to_tiberius_config();
        let log_settings = options.log_settings.clone();
        let cache_capacity = options.statement_cache_capacity;

        let handler = EstablishHandler { config };

        crate::net::connect_tcp(&options.host, options.port, handler)
            .await?
            .map(|client| MssqlConnection {
                inner: Box::new(MssqlConnectionInner {
                    client,
                    transaction_depth: 0,
                    pending_rollback: false,
                    log_settings,
                    cache_statement: StatementCache::new(cache_capacity),
                }),
            })
    }
}

struct EstablishHandler {
    config: tiberius::Config,
}

impl WithSocket for EstablishHandler {
    type Output = Result<tiberius::Client<SocketAdapter<Box<dyn Socket>>>, Error>;

    async fn with_socket<S: Socket>(self, socket: S) -> Self::Output {
        let boxed: Box<dyn Socket> = Box::new(socket);
        let adapter = SocketAdapter::new(boxed);
        tiberius::Client::connect(self.config, adapter)
            .await
            .map_err(tiberius_err)
    }
}
