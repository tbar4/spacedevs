use crate::schema::SchemaManager;
use reqwest::{Client, Error};
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

/// Response structure for paginated API endpoints
#[derive(Debug, Clone, serde::Deserialize)]
pub struct PaginatedResponse<T> {
    pub count: u32,
    pub next: Option<String>,
    pub previous: Option<String>,
    pub results: Vec<T>,
}

/// A generic REST API client that can work with any RESTful API
pub struct RESTClient {
    client: Arc<Client>,
    base_url: String,
    schema_manager: Option<SchemaManager>,
}

impl RESTClient {
    /// Create a new RESTClient with the specified base URL
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            client: Arc::new(Client::new()),
            base_url: base_url.into(),
            schema_manager: None,
        }
    }

    /// Create a new RESTClient with custom configuration
    pub fn with_client(base_url: impl Into<String>, client: Client) -> Self {
        Self {
            client: Arc::new(client),
            base_url: base_url.into(),
            schema_manager: None,
        }
    }

    /// Create a new RESTClient with schema support
    pub fn with_schemas(base_url: impl Into<String>, schema_manager: SchemaManager) -> Self {
        Self {
            client: Arc::new(Client::new()),
            base_url: base_url.into(),
            schema_manager: Some(schema_manager),
        }
    }

    /// Get a reference to the underlying reqwest client
    pub fn client(&self) -> &Client {
        &self.client
    }

    /// Build a full URL for an endpoint
    fn build_url(&self, endpoint: &str) -> String {
        format!(
            "{}/{}",
            self.base_url.trim_end_matches('/'),
            endpoint.trim_start_matches('/')
        )
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

    /// Fetch data from an endpoint and return raw JSON
    pub async fn get_json(&self, endpoint: &str) -> Result<Value, Error> {
        let url = self.build_url(endpoint);
        let response = self.client.get(&url).send().await?;
        response.json::<Value>().await
    }

    /// Fetch data from an endpoint and apply a schema to it
    pub async fn get_with_schema(
        &self,
        endpoint: &str,
        schema_name: &str,
    ) -> Result<Value, Box<dyn std::error::Error>> {
        if let Some(schema_manager) = &self.schema_manager {
            let json_data = self.get_json(endpoint).await?;
            schema_manager.apply_schema(schema_name, &json_data)
        } else {
            Err("No schema manager configured".into())
        }
    }

    /// Fetch data from an endpoint with query parameters defined in schema
    pub async fn get_with_params<T>(
        &self,
        endpoint: &str,
        schema_name: &str,
        params: &HashMap<String, String>,
    ) -> Result<T, Box<dyn std::error::Error>>
    where
        T: DeserializeOwned,
    {
        if let Some(schema_manager) = &self.schema_manager {
            let query_string = schema_manager.build_query_string(schema_name, params)?;
            let url = format!("{}{}", self.build_url(endpoint), query_string);
            let response = self.client.get(&url).send().await?;
            Ok(response.json::<T>().await?)
        } else {
            Err("No schema manager configured".into())
        }
    }

    /// Fetch data from an endpoint with query parameters and apply schema
    pub async fn get_with_params_and_schema(
        &self,
        endpoint: &str,
        schema_name: &str,
        params: &HashMap<String, String>,
    ) -> Result<Value, Box<dyn std::error::Error>> {
        if let Some(schema_manager) = &self.schema_manager {
            let query_string = schema_manager.build_query_string(schema_name, params)?;
            let url = format!("{}{}", self.build_url(endpoint), query_string);
            let response = self.client.get(&url).send().await?;
            let json_data = response.json::<Value>().await?;
            schema_manager.apply_schema(schema_name, &json_data)
        } else {
            Err("No schema manager configured".into())
        }
    }
}
