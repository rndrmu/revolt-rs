use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::id::{ChannelId, ServerId, UserId};


#[derive(Serialize, Deserialize, Debug, Clone)]

#[non_exhaustive]
pub enum ChannelType {
    TextChannel,
    VoiceChannel,
    SavedMessages,
    DirectMessage,
    Unknown,
}

#[derive(Serialize, Deserialize, Debug, Clone)]

pub struct Channel {
    pub channel_type: ChannelType,
    #[serde(rename = "_id")]
    pub channel_id: ChannelId,
    #[serde(rename = "server")]
    pub server_id: ServerId,
    pub name: String,
    pub description: Option<String>,
    pub icon: Option<ChannelIcon>,
    pub default_permissions: Option<ChannelDefaultPermissions>,
    pub last_message_id: Option<String>,
    pub nsfw: Option<bool>,
    #[serde(flatten)]
    pub role_permissions: Option<HashMap<String, ChannelDefaultPermissions>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChannelIcon {
    #[serde(rename = "_id")]
    pub icon_id: String,
    pub tag: String,
    pub filename: String,
    pub metadata: ChannelIconMetadata,
    pub content_type: String,
    pub size: i32,
    pub deleted: Option<bool>,
    pub reported: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChannelIconMetadata {
    #[serde(rename = "type")]
    pub file_type: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChannelDefaultPermissions {
    #[serde(rename = "a")]
    pub allow: i32,
    #[serde(rename = "d")]
    pub deny: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DMChannel {
    pub channel_type: ChannelType,
    #[serde(rename = "_id")]
    pub channel_id: ChannelId,
    pub active: Option<Option<bool>>,
    pub recipients: Option<Option<Vec<UserId>>>,
    pub user: Option<Option<UserId>>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Invite {
    #[serde(rename = "type")]
    pub target_type: Option<String>,
    #[serde(rename = "_id")]
    pub id: Option<String>,
    pub server: Option<ServerId>,
    pub creator: Option<UserId>,
    pub channel: Option<ChannelId>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PartialChannel {
    pub channel_type: Option<ChannelType>,
    #[serde(rename = "_id")]
    pub channel_id: Option<ChannelId>,
    pub server_id: Option<ServerId>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub icon: Option<ChannelIcon>,
    pub default_permissions: Option<ChannelDefaultPermissions>,
    pub last_message_id: Option<String>,
    pub nsfw: Option<bool>,
    #[serde(flatten)]
    pub role_permissions: Option<HashMap<String, ChannelDefaultPermissions>>,
}