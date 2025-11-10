use super::{author::Author, event::Event, launch::Launch};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Article {
    pub id: u32,
    pub title: String,
    pub url: String,
    pub image_url: String,
    pub news_site: String,
    pub summary: String,
    pub published_at: String,
    pub updated_at: String,
    pub featured: bool,
    #[serde(default)]
    pub authors: Vec<Author>,
    #[serde(default)]
    pub launches: Vec<Launch>,
    #[serde(default)]
    pub events: Vec<Event>,
}
