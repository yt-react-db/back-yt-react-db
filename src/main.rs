use actix_cors::Cors;
use actix_web::{get, web::Data, App, HttpResponse, HttpServer, Responder, middleware::Logger};
use env_logger::Env;
use reqwest::Client;
use routes::{google_routes::get_the_juice, data::{get_full_permissions_list, get_permission_by_channel_id}};
use routes::data::set_permissions;


pub mod config;
pub mod routes;
pub mod models;

#[get("/health_check")]
async fn health_check() -> impl Responder {
    HttpResponse::Ok().body("UP")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {

    env_logger::init_from_env(Env::default().default_filter_or("debug"));

    let config = config::AppConfig::new();
    let reqwest_client = Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .unwrap();
    let db_pool = config.init_connection_pool();


    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header();


        App::new()
            .wrap(cors)
            .wrap(Logger::default())
            .service(health_check)
            .service(get_the_juice)
            .service(set_permissions)
            .service(get_full_permissions_list)
            .service(get_permission_by_channel_id)
            .app_data(Data::new(reqwest_client.clone()))
            .app_data(Data::new(config.clone()))
            .app_data(Data::new(db_pool.clone()))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}