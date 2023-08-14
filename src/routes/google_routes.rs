
use actix_web::ResponseError;
use actix_web::http::header::ContentType;
use actix_web::{web, post, HttpResponse, http::StatusCode};
use log::error;
use secrecy::ExposeSecret;
use serde::Deserialize;
use reqwest::Client;
use anyhow::anyhow;
use anyhow::Context;
use serde_json::json;

use crate::config::AppConfig;
use crate::models::claim::ClaimPermissions;
use jwt_simple::prelude::*;


/// The Authorization code the user received in the frontend after signing
/// in with Google & giving (or not) consent.
#[derive(Deserialize, Debug)]
struct AuthCode {
    code: String,
}

#[derive(Deserialize, Debug)]
struct AccessToken {
    access_token: String,
    /* useless for my use case
    expires_in: u32,
    id_token: String,
    refresh_token: String,
    scope: String,
    token_type: String,
    */
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PageInfo {
    total_results: u32,
    //results_per_page: u32,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Channel {
    pub title: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BrandingSettings {
    pub channel: Channel,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Item {
    pub id: String,
    pub branding_settings: BrandingSettings,
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
pub struct ChannelsInfo {
    //kind: String,
    pub page_info: PageInfo,
    pub items: Vec<Item>,
}

// ERROR shit ------------------------------------------------------------------
#[derive(thiserror::Error, Debug)]
pub enum GError {
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),

    #[error("{0}")]
    UserError(String),
}

impl ResponseError for GError {
    fn status_code(&self) -> StatusCode {
        match self {
            GError::UnexpectedError(_) => {
                error!("{:?}", self);
                StatusCode::INTERNAL_SERVER_ERROR
            },
            GError::UserError(_) => {
                StatusCode::BAD_REQUEST
            }
        }
    }

    fn error_response(&self) -> HttpResponse {
        let message = match self {
            GError::UnexpectedError(_) => "Unexpected error, please try again",
            GError::UserError(msg) => &msg,
        };
        let json_response = json!({
            "message": message,
        }).to_string();
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::json())
            .body(json_response)
    }
}

// -----------------------------------------------------------------------------

/// get_access_token
/// 
/// Get access token of a google user using the authorization code received after
/// signing in with google.
/// This is the [Authorization Code flow](https://developers.google.com/identity/oauth2/web/guides/use-code-model)
async fn get_access_token(auth_code: &AuthCode, client: &Client, config: &AppConfig) -> Result<AccessToken, GError> {

    let redirect_uri= String::from("postmessage");
    let grant_type = String::from("authorization_code");

    let params = [
        ("code", &auth_code.code),
        ("client_id", &config.google.client_id),
        ("client_secret", &config.google.client_secret.expose_secret()),
        ("redirect_uri", &redirect_uri),
        ("grant_type", &grant_type)
    ];

    let res = client.post(&config.google.oauth2_token_url)
        .form(&params)
        .send()
        .await
        .map_err(|_| anyhow!("Unable to get access token"))?;

    if res.status() == 400 {
        return Err(GError::UserError("Bad request".into()));
    } else if res.status() != 200 {
        return Err(GError::UnexpectedError(anyhow!("Google is unhappy, unable to get access token")));
    }

    // TODO: check if scope is present

    let res = res.json::<AccessToken>().await.context("unable to parse access token")?;
    Ok(res)
}

/// Request the Youtube data v3 API for information about youtube channels of a
/// google user.
/// The goal here to access the title (name) and ID of 
async fn get_channels_info(token_response: &AccessToken, client:&Client, config: &AppConfig) -> Result<ChannelsInfo, GError> {

    let res = client.get(&config.google.youtube_channel_info_url)
        .header("Authorization", format!("Bearer {}", token_response.access_token))
        .send()
        .await
        .context("Unable to get channels information")?;

    // yes I reuse code, with ctrl+c, ctrl+v
    if res.status() == 400 {
        return Err(GError::UserError("Bad request".into()));
    } else if res.status() != 200 {
        return Err(GError::UnexpectedError(anyhow!("Google is unhappy, unable to get channels info")));
    }

    Ok(res.json::<ChannelsInfo>().await.context("unable to parse channels info")?)
}

/// Returns a jwt containing channel_id & channel_title in the body.
/// A safer approach would be to set the jwt in a HttpOnly cookie, and put the
/// channel_id & channel_title in the body. 
#[post("/get_the_juice")]
async fn get_the_juice(data: web::Json<AuthCode>, client: web::Data<Client>, config: web::Data<AppConfig>) -> Result<HttpResponse, GError> {


    let token_response = get_access_token(&data, &client, &config).await?;
    let channels_info = get_channels_info(&token_response, &client, &config).await?;

    if channels_info.page_info.total_results == 0 {
       return Err(GError::UnexpectedError(anyhow!("No channels found")));
    }
    // can we have more than 1? not in our case, right? if it causes a problem
    // surely someone will hit me up

    // these names are trash
    let claim_permissions = ClaimPermissions::new(&channels_info);
    let claim = Claims::with_custom_claims(claim_permissions, Duration::from_hours(1));
    let token = config.key.authenticate(claim).context("Failed to create token")?;

    Ok(HttpResponse::Ok().body(
        json!({
            "message": token,
        }).to_string()
    ))
}
