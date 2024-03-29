use serde::{Serialize, Deserialize};
use crate::routes::google_routes::ChannelsInfo;

#[derive(Serialize, Deserialize, Debug)]
pub struct ClaimPermissions {
    pub channel_id: String,
    pub channel_title: String,
    pub custom_url: String,
}

impl ClaimPermissions {
    pub fn new(channels_info: &ChannelsInfo) -> Self {
        ClaimPermissions {
            channel_id: channels_info.items[0].id.clone(),
            channel_title: channels_info.items[0].snippet.title.clone(),
            custom_url: channels_info.items[0].snippet.custom_url.clone(),
        }
    }
}