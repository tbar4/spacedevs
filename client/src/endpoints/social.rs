use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Social {
    #[serde(rename = "x")]
    pub twitter: Option<String>,
    pub youtube: Option<String>,
    pub instagram: Option<String>,
    pub linkedin: Option<String>,
    #[serde(rename = "mastodon")]
    pub mastodon: Option<String>,
    pub bluesky: Option<String>,
}
