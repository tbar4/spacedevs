use crate::endpoints::{article::Article, blog::Blog, report::Report};
use crate::utils::urls::*;
use reqwest::{Client, Error};
use serde::de::DeserializeOwned;
use std::sync::Arc;

/// Response structure for paginated API endpoints
#[derive(Debug, Clone, serde::Deserialize)]
pub struct PaginatedResponse<T> {
    pub count: u32,
    pub next: Option<String>,
    pub previous: Option<String>,
    pub results: Vec<T>,
}

pub struct SpaceDevsClient {
    client: Arc<Client>,
    base_url: String,
}

pub struct SpaceDevsDataClient {
    client: Arc<Client>,
    base_url: String,
}

impl SpaceDevsClient {
    /// Create a new SpaceDevsClient with default configuration
    pub fn new() -> Self {
        Self {
            client: Arc::new(Client::new()),
            base_url: SPACEFLIGHT_NEWS_API_BASE.to_string(),
        }
    }

    /// Create a new SpaceDevsClient with custom base URL (useful for testing)
    pub fn with_base_url(base_url: String) -> Self {
        Self {
            client: Arc::new(Client::new()),
            base_url,
        }
    }

    /// Create a new SpaceDevsClient with custom configuration
    pub fn with_client(client: Client) -> Self {
        Self {
            client: Arc::new(client),
            base_url: SPACEFLIGHT_NEWS_API_BASE.to_string(),
        }
    }

    /// Get a reference to the underlying reqwest client
    pub fn client(&self) -> &Client {
        &self.client
    }

    /// Build a full URL for an endpoint
    fn build_url(&self, endpoint: &str) -> String {
        format!("{}/{}", self.base_url, endpoint.trim_start_matches('/'))
    }

    /// Fetch data from an endpoint and deserialize it
    pub async fn get<T>(&self, endpoint: &str) -> Result<T, Error>
    where
        T: DeserializeOwned,
    {
        let url = self.build_url(endpoint);
        let response = self.client.get(&url).send().await?;
        response.json::<T>().await
    }

    /// Fetch articles with structured data
    pub async fn get_articles_structured(&self) -> Result<PaginatedResponse<Article>, Error> {
        self.get("articles").await
    }

    /// Fetch blogs with structured data
    pub async fn get_blogs_structured(&self) -> Result<PaginatedResponse<Blog>, Error> {
        self.get("blogs").await
    }

    /// Fetch reports with structured data
    pub async fn get_reports_structured(&self) -> Result<PaginatedResponse<Report>, Error> {
        self.get("reports").await
    }

    /// Fetch a single article by ID
    pub async fn get_article(&self, id: u32) -> Result<Article, Error> {
        self.get(&format!("articles/{}", id)).await
    }

    /// Fetch a single blog by ID
    pub async fn get_blog(&self, id: u32) -> Result<Blog, Error> {
        self.get(&format!("blogs/{}", id)).await
    }

    /// Fetch a single report by ID
    pub async fn get_report(&self, id: u32) -> Result<Report, Error> {
        self.get(&format!("reports/{}", id)).await
    }

    /// Fetch articles endpoint (raw JSON)
    pub async fn get_articles(&self) -> Result<serde_json::Value, Error> {
        self.get("articles").await
    }

    /// Fetch blogs endpoint (raw JSON)
    pub async fn get_blogs(&self) -> Result<serde_json::Value, Error> {
        self.get("blogs").await
    }

    /// Fetch reports endpoint (raw JSON)
    pub async fn get_reports(&self) -> Result<serde_json::Value, Error> {
        self.get("reports").await
    }
}

// Implementation for SpaceDevsDataClient
impl SpaceDevsDataClient {
    /// Create a new SpaceDevsDataClient with default configuration
    pub fn new() -> Self {
        Self {
            client: Arc::new(Client::new()),
            base_url: SPACEDEVS_DATA_API_BASE.to_string(),
        }
    }

    /// Create a new SpaceDevsDataClient with custom base URL
    pub fn with_base_url(base_url: String) -> Self {
        Self {
            client: Arc::new(Client::new()),
            base_url,
        }
    }

    /// Get a reference to the underlying reqwest client
    pub fn client(&self) -> &Client {
        &self.client
    }

    /// Build a full URL for an endpoint
    fn build_url(&self, endpoint: &str) -> String {
        format!("{}/{}", self.base_url, endpoint.trim_start_matches('/'))
    }

    /// Fetch data from an endpoint and deserialize it
    pub async fn get<T>(&self, endpoint: &str) -> Result<T, Error>
    where
        T: DeserializeOwned,
    {
        let url = self.build_url(endpoint);
        let response = self.client.get(&url).send().await?;
        response.json::<T>().await
    }
}
