use actix_web::{post, web, HttpResponse, Result, ResponseError};
use anyhow::Context;
use jwt_simple::prelude::MACLike;
use serde::Deserialize;

use crate::{models::claim::ClaimPermissions, config::AppConfig};


#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Permission {
    Yes,
    YesWithDelay,
    No,
}

#[derive(Debug, Deserialize)]
struct YoutuberPermissions {

    channel_id: String,
    channel_title: String,

    can_react_live: Permission,
    live_reaction_delay: String,

    can_upload_reaction: Permission,
    upload_reaction_delay: String,

    last_updated_at: String,

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
pub enum PermissionsError {
    #[error("TODO error handling")]
    ToDo(#[from] anyhow::Error),
}

impl ResponseError for PermissionsError {}

#[post("/set_permissions")]
pub async fn set_permissions(permissions: web::Json<SetPermissionsData>, config: web::Data<AppConfig>) -> Result<HttpResponse, PermissionsError> {

    let claims = config.key.verify_token::<ClaimPermissions>(&permissions.token, None)
        .context("failed to verify token")?;
    println!("{:?}", claims);
    println!("{:?}", permissions);

    Ok(HttpResponse::Ok().body("ok"))
}