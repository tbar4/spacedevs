use super::author::Author;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Report {
    pub id: u32,
    pub title: String,
    #[serde(default)]
    pub authors: Vec<Author>,
    pub url: String,
    pub image_url: String,
    pub news_site: String,
    #[serde(default)]
    pub summary: Option<String>,
    pub published_at: String,
    pub updated_at: String,
}
