use config::{File, FileFormat, Environment};
use secrecy::Secret;
use serde::Deserialize;
use serde_aux::prelude::deserialize_number_from_string;
use dotenv::dotenv;

#[derive(Deserialize, Clone, Debug)]
pub struct AppConfig {
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
}

pub fn init_config() -> AppConfig {

    dotenv().ok();

    let builder = config::Config::builder()
        .add_source(Environment::default())
        .add_source(File::new("config/config.yaml", FileFormat::Yaml))
        .build()
        .expect("Failed to create config");
    
    builder.try_deserialize::<AppConfig>().expect("Failed to deserialize config")

}
