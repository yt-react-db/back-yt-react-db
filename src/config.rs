use config::{File, FileFormat, Environment};
use secrecy::{Secret, ExposeSecret};
use serde::Deserialize;
use serde_aux::prelude::deserialize_number_from_string;
use dotenv::dotenv;
use jwt_simple::prelude::*;
use sqlx::{postgres::{PgConnectOptions, PgSslMode, PgPoolOptions}, ConnectOptions, PgPool};

#[derive(Deserialize, Clone, Debug)]
pub struct PartialAppConfig {
    pub google: GoogleConfig,
    pub database: DatabaseConfig,
}

#[derive(Deserialize, Clone, Debug)]
pub struct GoogleConfig {
    pub client_id: String,
    pub client_secret: Secret<String>,
    pub oauth2_token_url: String,
    pub youtube_channel_info_url: String,
}

#[derive(Deserialize, Clone, Debug)]
pub struct DatabaseConfig {
    pub host: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub database_name: String,
    pub username: String,
    pub password: Secret<String>,
    pub require_ssl: bool,
}

/*
// could use this to read .env file
#[derive(Deserialize, Clone, Debug)]
struct MySource;

impl Source for MySource {
    fn clone_into_box(&self) -> Box<dyn Source + Send + Sync> {
        Box::new((*self).clone())
    }
    fn collect(&self) -> Result<Map<String, Value>, ConfigError> {
        let mut map = Map::new();
        map.insert("test".to_string(), Value::new(Some(&"test_origin".to_string()), ValueKind::String("your_mom".to_string())));
        Ok(map)
    }
}
*/


#[derive(Clone, Debug)]
pub struct AppConfig {
    pub google: GoogleConfig,
    pub database: DatabaseConfig,
    pub key: HS256Key,
}

impl AppConfig {

    pub fn new() -> AppConfig {

        dotenv().ok();

        let builder = config::Config::builder()
            .add_source(Environment::default())
            .add_source(File::new("config/config.yaml", FileFormat::Yaml))
            .build()
            .expect("Failed to create config");
        let partial = builder.try_deserialize::<PartialAppConfig>().expect("Failed to deserialize config");

        AppConfig {
            google: partial.google,
            database: partial.database,
            /// yep, a new key is generated every time the app starts, shouldn't be a big deal
            /// If you have an unlucky timing, you will have to redo the process, my bad, you will be fine
            key: HS256Key::generate(),
        }

    }

    pub fn init_connection_pool(&self) -> PgPool {
        let ssl_mode = if self.database.require_ssl {
            PgSslMode::Require
        } else {
            PgSslMode::Prefer
        };
        let pg_conn_options = PgConnectOptions::new()
            .host(&self.database.host)
            .username(&self.database.username)
            .password(self.database.password.expose_secret())
            .port(self.database.port)
            .ssl_mode(ssl_mode)
            .log_statements(log::LevelFilter::Trace) // TODO: change this
            .database(&self.database.database_name);
        
        PgPoolOptions::new()
            .acquire_timeout(std::time::Duration::from_secs(5))
            .max_connections(10)
            .connect_lazy_with(pg_conn_options)
    }

}