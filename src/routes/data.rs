use actix_web::{post, web::{self}, HttpResponse, Result, ResponseError, Responder, get, http::{StatusCode, header::{CacheControl, CacheDirective, ContentType}}};
use anyhow::Context;
use jwt_simple::prelude::MACLike;
use log::debug;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, types::chrono::{DateTime, Utc}};

use crate::{models::claim::ClaimPermissions, config::AppConfig};


#[derive(Debug, Deserialize, Serialize, sqlx::Type, Clone)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name="permission", rename_all = "snake_case")]
enum Permission {
    Yes,
    YesWithDelay,
    No,
}

#[derive(Debug, Serialize)]
struct YoutuberPermissions {

    channel_id: String,
    channel_title: String,

    can_react_live: Permission,
    live_reaction_delay: Option<String>,

    can_upload_reaction: Permission,
    upload_reaction_delay: Option<String>,

    last_updated_at: DateTime<Utc>,

}

#[derive(Debug, Deserialize)]
pub struct SetPermissionsData {
    can_react_live: Permission,
    live_reaction_delay_value: u16,
    live_reaction_delay_unit: String,

    can_upload_reaction: Permission,
    upload_reaction_delay_value: u16,
    upload_reaction_delay_unit: String,

    token: String,
}

#[derive(Debug, thiserror::Error)]
pub enum DataError {
    #[error("TODO error handling")]
    ToDo(#[from] anyhow::Error),

    #[error("{0}")]
    NotFound(String)
}

impl ResponseError for DataError {
    fn status_code(&self) -> reqwest::StatusCode {
        match *self {
            DataError::NotFound(_) => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse<actix_web::body::BoxBody> {
        match *self {
            DataError::NotFound(ref msg) => HttpResponse::build(self.status_code())
                .insert_header(CacheControl(vec![CacheDirective::MaxAge(86400u32)])) // one day (maybe could be more)
                .insert_header(ContentType::json())
                .body(msg.clone())
            ,
            _ => HttpResponse::build(self.status_code())
                .insert_header(ContentType::plaintext())
                .body(self.to_string())
        }
    }
}

#[post("/set_permissions")]
pub async fn set_permissions(permissions: web::Json<SetPermissionsData>, config: web::Data<AppConfig>, conn: web::Data<PgPool>) -> Result<HttpResponse, DataError> {

    let claims = config.key.verify_token::<ClaimPermissions>(&permissions.token, None)
        .context("failed to verify token")?.custom;
    println!("{:?}", claims);
    println!("{:?}", permissions);

    // extract this

    // TODO: add some value checking (unit = d, w, m, y), or enforce it with an enum?
    // TODO: check value of "can_..." if != than yes_with_delay, set delay to NULL
    let live_reaction_delay   = format!("{}{}", permissions.live_reaction_delay_value,   permissions.live_reaction_delay_unit);
    let upload_reaction_delay = format!("{}{}", permissions.upload_reaction_delay_value, permissions.upload_reaction_delay_unit);

    
    // extract this
    sqlx::query!(
        "INSERT INTO youtuber_permissions \
            (channel_id, channel_title, custom_url,\
            can_react_live, live_reaction_delay, \
            can_upload_reaction, upload_reaction_delay) \
        VALUES ($1, $2, $3, $4, $5, $6, $7) \
        ON CONFLICT (channel_id) DO UPDATE \
            SET can_react_live = EXCLUDED.can_react_live, \
            live_reaction_delay = EXCLUDED.live_reaction_delay, \
            can_upload_reaction = EXCLUDED.can_upload_reaction, \
            upload_reaction_delay = EXCLUDED.upload_reaction_delay, \
            channel_title = EXCLUDED.channel_title, \
            custom_url = EXCLUDED.custom_url, \
            last_updated_at = NOW(); \
        ",
        claims.channel_id, claims.channel_title, claims.custom_url,
        permissions.can_react_live.clone() as Permission, live_reaction_delay, 
        permissions.can_upload_reaction.clone() as Permission, upload_reaction_delay, 

    ).execute(conn.get_ref()).await.context("couldn't insert permissions into db")?;

    Ok(HttpResponse::Ok().body("ok"))
}


#[get("/permissions/full_list")]
pub async fn get_full_permissions_list(conn: web::Data<PgPool>) -> Result<impl Responder, DataError> {
    let full_list = sqlx::query_as!(YoutuberPermissions,
        r#"
        SELECT
            channel_id,
            channel_title,
            can_react_live as "can_react_live!: Permission",
            live_reaction_delay,
            can_upload_reaction as "can_upload_reaction!: Permission",
            upload_reaction_delay,
            last_updated_at as "last_updated_at!: DateTime<Utc>"
        FROM youtuber_permissions"#
    ).fetch_all(conn.get_ref()).await.context("couldn't fetch list")?;

    Ok(web::Json(full_list))
}


#[get("/permissions/{channel_id_or_custom_url}")]
pub async fn get_permission_by_channel_id(path: web::Path<String>, conn: web::Data<PgPool>) -> Result<impl Responder, DataError> {

    debug!("again?");
    let channel_id_or_custom_url = path.into_inner();

    let permissions;

    if channel_id_or_custom_url.starts_with("@") {

        permissions = sqlx::query_as!(YoutuberPermissions,
            r#"
            SELECT
                channel_id,
                channel_title,
                can_react_live as "can_react_live!: Permission",
                live_reaction_delay,
                can_upload_reaction as "can_upload_reaction!: Permission",
                upload_reaction_delay,
                last_updated_at as "last_updated_at!: DateTime<Utc>"
            FROM youtuber_permissions
            WHERE LOWER(custom_url) = LOWER($1)"#,
            channel_id_or_custom_url
        ).fetch_optional(conn.get_ref()).await.context("couldn't fetch permissions")?;


    } else {

        permissions = sqlx::query_as!(YoutuberPermissions,
            r#"
            SELECT
                channel_id,
                channel_title,
                can_react_live as "can_react_live!: Permission",
                live_reaction_delay,
                can_upload_reaction as "can_upload_reaction!: Permission",
                upload_reaction_delay,
                last_updated_at as "last_updated_at!: DateTime<Utc>"
            FROM youtuber_permissions
            WHERE channel_id = $1"#,
            channel_id_or_custom_url
        ).fetch_optional(conn.get_ref()).await.context("couldn't fetch permissions")?;

    }

    debug!("{:?}", permissions);
    if let Some(permissions) = permissions {
        let response = HttpResponse::Ok()
            .insert_header(ContentType::json())
            .insert_header(CacheControl(vec![CacheDirective::MaxAge(86400u32)])) // one day (maybe could be more)
            // .insert_header(CacheControl(vec![CacheDirective::MaxAge(5u32)])) // TEST: use this line for testing purposes
            .insert_header(("Last-Modified", permissions.last_updated_at.to_string()))
            //.insert_header(("ETag", "abctest")) // use this?
            .json(permissions);
        Ok(response)
    } else {
        Err(DataError::NotFound(r#"{"message": "Permission not found for given channel_ID"}"#.to_string()))
    }
}
