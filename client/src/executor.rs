//! API Executor that reads configuration from TOML and executes API calls
//!
//! This module provides functionality to automatically execute API calls
//! based on configuration defined in a TOML file, without requiring any
//! Rust code changes.

use crate::RESTClient;
use crate::schema::SchemaManager;
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use toml::Value as TomlValue;

/// Configuration for a single API endpoint
#[derive(Debug, Clone)]
pub struct EndpointConfig {
    /// Name of the endpoint
    pub name: String,
    /// URL of the endpoint
    pub url: String,
    /// Whether this endpoint is enabled
    pub enabled: bool,
    /// Schema name for this endpoint
    pub schema_name: String,
    /// Query parameters for this endpoint
    pub query_params: HashMap<String, String>,
}

/// Global configuration
#[derive(Debug, Clone)]
pub struct GlobalConfig {
    /// Output format: "json", "table", or "detailed"
    pub output_format: String,
    /// Maximum number of items to display per endpoint
    pub max_display_items: usize,
}

/// API Executor that runs configurations from TOML files
pub struct APIExecutor {
    /// Schema manager for handling data schemas
    schema_manager: SchemaManager,
    /// Endpoint configurations
    endpoints: Vec<EndpointConfig>,
    /// Global configuration
    global_config: GlobalConfig,
}

impl APIExecutor {
    /// Create a new API executor from a TOML configuration file
    pub fn from_config_file(config_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let contents = fs::read_to_string(config_path)?;
        let config: TomlValue = toml::from_str(&contents)?;

        // Create a schema manager and load schemas from the same config
        let mut schema_manager = SchemaManager::new();
        schema_manager.load_from_toml_value(&config)?;

        // Parse endpoint configurations
        let endpoints = Self::parse_endpoints(&config)?;

        // Parse global configuration
        let global_config = Self::parse_global_config(&config)?;

        Ok(Self {
            schema_manager,
            endpoints,
            global_config,
        })
    }

    /// Parse endpoint configurations from TOML
    fn parse_endpoints(
        config: &TomlValue,
    ) -> Result<Vec<EndpointConfig>, Box<dyn std::error::Error>> {
        let mut endpoints = Vec::new();

        if let Some(tables) = config.as_table() {
            for (name, value) in tables {
                // Skip special sections
                if name == "config" || name == "types" {
                    continue;
                }

                // Skip schema-related sections
                if name.contains(".schema") || name.contains(".query_params") {
                    continue;
                }

                // Parse endpoint configuration
                if let Some(endpoint_table) = value.as_table() {
                    let enabled = endpoint_table
                        .get("enabled")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false);

                    if let Some(url) = endpoint_table.get("url").and_then(|v| v.as_str()) {
                        // Parse schema name (same as endpoint name)
                        let schema_name = name.clone();

                        // Parse query parameters
                        let mut query_params = HashMap::new();
                        let query_key = format!("{}.query_params", name);
                        if let Some(query_section) = tables.get(&query_key) {
                            if let Some(query_table) = query_section.as_table() {
                                for (param_name, param_value) in query_table {
                                    if let Some(value_str) = Self::toml_value_to_string(param_value)
                                    {
                                        query_params.insert(param_name.clone(), value_str);
                                    }
                                }
                            }
                        }

                        endpoints.push(EndpointConfig {
                            name: name.clone(),
                            url: url.to_string(),
                            enabled,
                            schema_name,
                            query_params,
                        });
                    }
                }
            }
        }

        Ok(endpoints)
    }

    /// Convert TOML value to string
    fn toml_value_to_string(value: &TomlValue) -> Option<String> {
        match value {
            TomlValue::String(s) => Some(s.clone()),
            TomlValue::Integer(i) => Some(i.to_string()),
            TomlValue::Float(f) => Some(f.to_string()),
            TomlValue::Boolean(b) => Some(b.to_string()),
            _ => None,
        }
    }

    /// Parse global configuration
    fn parse_global_config(config: &TomlValue) -> Result<GlobalConfig, Box<dyn std::error::Error>> {
        let output_format =
            if let Some(config_table) = config.get("config").and_then(|v| v.as_table()) {
                config_table
                    .get("output_format")
                    .and_then(|v| v.as_str())
                    .unwrap_or("detailed")
                    .to_string()
            } else {
                "detailed".to_string()
            };

        let max_display_items =
            if let Some(config_table) = config.get("config").and_then(|v| v.as_table()) {
                config_table
                    .get("max_display_items")
                    .and_then(|v| v.as_integer())
                    .unwrap_or(10) as usize
            } else {
                10
            };

        Ok(GlobalConfig {
            output_format,
            max_display_items,
        })
    }

    /// Execute all enabled endpoints
    pub async fn execute_all(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Executing API endpoints...\n");

        for endpoint in &self.endpoints {
            if endpoint.enabled {
                self.execute_endpoint(endpoint).await?;
            }
        }

        Ok(())
    }

    /// Execute a single endpoint
    async fn execute_endpoint(
        &self,
        _endpoint: &EndpointConfig,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("Fetching data from: {} ({})", _endpoint.name, _endpoint.url);

        // Extract base URL and endpoint path
        let (base_url, endpoint_path) = Self::split_url(&_endpoint.url);

        // Create a new client with the correct base URL for this endpoint
        let client = RESTClient::with_schemas(&base_url, self.schema_manager.clone());

        // Execute the request
        match client
            .get_with_params_and_schema(
                &endpoint_path,
                &_endpoint.schema_name,
                &_endpoint.query_params,
            )
            .await
        {
            Ok(data) => {
                self.display_results(_endpoint, &data)?;
            }
            Err(e) => {
                eprintln!("Error fetching {}: {}", _endpoint.name, e);
            }
        }

        println!(); // Add spacing between endpoints
        Ok(())
    }

    /// Split URL into base URL and endpoint path
    fn split_url(url: &str) -> (String, String) {
        if let Some(last_slash) = url.rfind('/') {
            let base_url = &url[..last_slash];
            let endpoint = &url[last_slash + 1..];
            (base_url.to_string(), endpoint.to_string())
        } else {
            (url.to_string(), "".to_string())
        }
    }

    /// Display results based on configuration
    fn display_results(
        &self,
        _endpoint: &EndpointConfig,
        data: &Value,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match self.global_config.output_format.as_str() {
            "json" => {
                println!("{}", serde_json::to_string_pretty(data)?);
            }
            "table" => {
                self.display_as_table(_endpoint, data)?;
            }
            "detailed" | _ => {
                self.display_detailed(_endpoint, data)?;
            }
        }

        Ok(())
    }

    /// Display results in detailed format
    fn display_detailed(
        &self,
        _endpoint: &EndpointConfig,
        data: &Value,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(obj) = data.as_object() {
            if obj.contains_key("results") && obj.contains_key("count") {
                // Paginated response
                if let Some(count) = obj.get("count").and_then(|v| v.as_u64()) {
                    println!("  Total results: {}", count);
                }

                if let Some(results) = obj.get("results").and_then(|v| v.as_array()) {
                    println!(
                        "  Displaying first {} items:",
                        std::cmp::min(results.len(), self.global_config.max_display_items)
                    );

                    for (i, item) in results
                        .iter()
                        .take(self.global_config.max_display_items)
                        .enumerate()
                    {
                        println!("    Item {}:", i + 1);
                        self.display_object(item, 6)?;
                    }
                }
            } else {
                // Single object
                println!("  Response:");
                self.display_object(data, 4)?;
            }
        } else {
            println!("  Response: {:?}", data);
        }

        Ok(())
    }

    /// Display a JSON object with indentation
    fn display_object(
        &self,
        value: &Value,
        indent: usize,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let indent_str = " ".repeat(indent);

        if let Some(obj) = value.as_object() {
            for (key, val) in obj {
                match val {
                    Value::Object(_) => {
                        println!("{}{}:", indent_str, key);
                        self.display_object(val, indent + 2)?;
                    }
                    Value::Array(arr) => {
                        println!("{}{}: [{} items]", indent_str, key, arr.len());
                        if !arr.is_empty() && key != "events" && key != "launches" {
                            if let Some(first) = arr.first() {
                                if first.is_object() {
                                    println!("{}  First item:", indent_str);
                                    self.display_object(first, indent + 4)?;
                                } else {
                                    println!("{}  First item: {:?}", indent_str, first);
                                }
                            }
                        }
                    }
                    _ => {
                        println!("{}{}: {}", indent_str, key, val);
                    }
                }
            }
        } else {
            println!("{}{:?}", indent_str, value);
        }

        Ok(())
    }

    /// Display results in table format
    fn display_as_table(
        &self,
        _endpoint: &EndpointConfig,
        data: &Value,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(obj) = data.as_object() {
            if let Some(results) = obj.get("results").and_then(|v| v.as_array()) {
                println!(
                    "  | {:<30} | {:<20} | {:<20} |",
                    "Title", "News Site", "Published"
                );
                println!("  |{:-<32}|{:-<22}|{:-<22}|", "", "", "");

                for item in results.iter().take(self.global_config.max_display_items) {
                    if let Some(item_obj) = item.as_object() {
                        let title = item_obj
                            .get("title")
                            .and_then(|v| v.as_str())
                            .unwrap_or("N/A");
                        let news_site = item_obj
                            .get("news_site")
                            .and_then(|v| v.as_str())
                            .unwrap_or("N/A");
                        let published = item_obj
                            .get("published_at")
                            .and_then(|v| v.as_str())
                            .unwrap_or("N/A");

                        // Truncate long titles
                        let title_truncated = if title.len() > 27 {
                            format!("{}...", &title[..27])
                        } else {
                            title.to_string()
                        };

                        println!(
                            "  | {:<30} | {:<20} | {:<20} |",
                            title_truncated,
                            news_site,
                            &published[..std::cmp::min(20, published.len())]
                        );
                    }
                }
            }
        }

        Ok(())
    }
}

impl Default for APIExecutor {
    fn default() -> Self {
        Self {
            schema_manager: SchemaManager::new(),
            endpoints: Vec::new(),
            global_config: GlobalConfig {
                output_format: "detailed".to_string(),
                max_display_items: 10,
            },
        }
    }
}
