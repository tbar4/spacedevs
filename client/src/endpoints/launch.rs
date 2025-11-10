use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Launch {
    #[serde(rename = "launch_id")]
    pub id: String,
    pub provider: String,
}
