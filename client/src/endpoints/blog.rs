use super::{author::Author, event::Event, launch::Launch};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Blog {
    pub id: u32,
    pub title: String,
    #[serde(default)]
    pub authors: Vec<Author>,
    pub url: String,
    pub image_url: String,
    pub news_site: String,
    pub summary: String,
    pub published_at: String,
    pub updated_at: String,
    pub featured: bool,
    #[serde(default)]
    pub launches: Vec<Launch>,
    #[serde(default)]
    pub events: Vec<Event>,
}
