use actix_cors::Cors;
use actix_web::{get, post, web::{self, Data}, App, HttpResponse, HttpServer, Responder, middleware::Logger};
use env_logger::Env;
use log::info;
use reqwest::Client;
use secrecy::ExposeSecret;
use serde::Deserialize;

use crate::config::AppConfig;

pub mod config;

#[get("/health_check")]
async fn health_check() -> impl Responder {
    HttpResponse::Ok().body("UP")
}

#[post("/")]
async fn handle_post(data: web::Json<serde_json::Value>) -> impl Responder {
    println!("{}", data);
    HttpResponse::Ok()
}

#[derive(Deserialize, Debug)]
struct AuthCode {
    code: String,
}

#[derive(Deserialize, Debug)]
struct AccessToken {
    access_token: String,
    /*
    expires_in: u32,
    id_token: String,
    refresh_token: String,
    scope: String,
    token_type: String,
    */
}


#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct PageInfo {
    total_results: u32,
    results_per_page: u32,
}
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Channel {
    title: String,
}
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct BrandingSettings {
    channel: Channel,
}
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Item {
    id: String,
    branding_settings: BrandingSettings,
}

/// Example
/// ```JSON
/// {
///  "kind": "youtube#channelListResponse",
///  "etag": "whatever_this_is",
///  "pageInfo": {
///    "totalResults": 1,
///    "resultsPerPage": 5
///  },
///  "items": [
///    {
///      "kind": "youtube#channel",
///      "etag": "oijefa",
///      "id": "UCIv6GIlP5uXbiu666bOUobQ",
///      "brandingSettings": {
///        "channel": {
///          "title": "ComputerBread",
///          "description": "oiajf",
///          "keywords": "...",
///          "country": "FR"
///        },
///        "image": {
///          "bannerExternalUrl": "https://yt3.googleusercontent.com/58..."
///        }
///      }
///    }
///  ]
///}
/// ```
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct ChannelsInfo {
    kind: String,
    page_info: PageInfo,
    items: Vec<Item>,
}

#[post("/get_the_juice")]
async fn get_the_juice(data: web::Json<AuthCode>, client: web::Data<Client>, config: web::Data<AppConfig>) -> impl Responder {

    let redirect_uri= String::from("postmessage");
    let grant_type = String::from("authorization_code");

    let params = [
        ("code", &data.code),
        ("client_id", &config.google.client_id),
        ("client_secret", &config.google.client_secret.expose_secret()),
        ("redirect_uri", &redirect_uri),
        ("grant_type", &grant_type)
    ];

    let res = client.post(&config.google.oauth2_token_url)
        .form(&params)
        .send()
        .await
        .unwrap();
    let token_response = res.json::<AccessToken>().await.unwrap();
    //let token_response = res.json::<serde_json::Value>().await.unwrap();
    info!("{:?}", token_response);

    let res = client.get(&config.google.youtube_channel_info_url)
        .header("Authorization", format!("Bearer {}", token_response.access_token))
        .send()
        .await
        .unwrap();
    let res = res.json::<ChannelsInfo>().await.unwrap();
    info!("{:?}", res);
    if res.page_info.total_results > 0 {
        info!("Channel title: {:?}", res.items[0].branding_settings.channel.title);
        info!("Channel ID: {:?}", res.items[0].id);
    }

    HttpResponse::Ok()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {

    let config = config::init_config();
    let reqwest_client = Client::new();

    env_logger::init_from_env(Env::default().default_filter_or("debug"));

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header();


        App::new()
            .wrap(cors)
            .wrap(Logger::default())
            .service(health_check)
            .service(handle_post)
            .service(get_the_juice)
            .app_data(Data::new(reqwest_client.clone()))
            .app_data(Data::new(config.clone()))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}