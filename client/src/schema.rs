//! Schema management for dynamic endpoint structures
//!
//! This module provides functionality to load struct definitions from TOML files
//! and use them to dynamically process API responses and build query parameters.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use urlencoding;

/// Represents a field in a struct definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldDefinition {
    /// Name of the field
    pub name: String,
    /// Type of the field (e.g., "String", "u32", "Vec<Author>")
    pub type_name: String,
    /// Whether this field is optional
    pub optional: bool,
}

/// Represents a query parameter definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryParamDefinition {
    /// Name of the parameter
    pub name: String,
    /// Type of the parameter
    #[serde(rename = "type")]
    pub param_type: String,
    /// Default value (if any)
    #[serde(default)]
    pub default: Option<QueryParamValue>,
    /// Description of the parameter
    #[serde(default)]
    pub description: Option<String>,
}

/// Possible values for query parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum QueryParamValue {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
}

impl QueryParamValue {
    /// Convert to string representation for URL encoding
    pub fn to_string(&self) -> String {
        match self {
            QueryParamValue::String(s) => s.clone(),
            QueryParamValue::Integer(i) => i.to_string(),
            QueryParamValue::Float(f) => f.to_string(),
            QueryParamValue::Boolean(b) => b.to_string(),
        }
    }
}

/// Represents a complete struct schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schema {
    /// Name of the struct
    pub name: String,
    /// Fields in the struct
    pub fields: Vec<FieldDefinition>,
    /// Nested fields that reference other schemas
    #[serde(default)]
    pub nested_fields: HashMap<String, String>,
    /// Supported query parameters
    #[serde(default)]
    pub query_params: HashMap<String, QueryParamDefinition>,
}

/// Schema manager that loads and manages struct definitions
#[derive(Debug, Clone)]
pub struct SchemaManager {
    /// Loaded schemas by name
    schemas: HashMap<String, Schema>,
}

impl SchemaManager {
    /// Create a new schema manager
    pub fn new() -> Self {
        Self {
            schemas: HashMap::new(),
        }
    }

    /// Load schemas from a TOML file
    pub fn load_from_file(&mut self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let contents = fs::read_to_string(path)?;
        let config: toml::Value = toml::from_str(&contents)?;
        self.load_from_toml_value(&config)
    }

    /// Load schemas from a TOML value
    pub fn load_from_toml_value(
        &mut self,
        config: &toml::Value,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Parse schemas from the TOML structure
        if let Some(tables) = config.as_table() {
            for (name, value) in tables {
                // Skip special sections like [types] and [config]
                if name == "types" || name == "config" {
                    continue;
                }

                // Skip query_params sections (they're processed as part of the main schema)
                if name.contains(".query_params") {
                    continue;
                }

                // Skip nested_fields sections (they're processed as part of the main schema)
                if name.contains(".nested_fields") {
                    continue;
                }

                // Skip schema sections (they're processed as part of the main schema)
                if name.contains(".schema") {
                    continue;
                }

                // Parse the schema
                if let Some(_schema_table) = value.as_table() {
                    // Get schema definition from the dedicated schema section
                    let schema_section_name = format!("{}.schema", name);
                    let (fields, nested_fields) = if let Some(schema_section) =
                        tables.get(&schema_section_name)
                    {
                        if let Some(schema_def_table) = schema_section.as_table() {
                            let mut fields = Vec::new();
                            let mut nested_fields = HashMap::new();

                            // Parse regular fields
                            for (field_name, field_type) in schema_def_table {
                                if field_name != "nested_fields" {
                                    if let Some(type_str) = field_type.as_str() {
                                        fields.push(FieldDefinition {
                                            name: field_name.clone(),
                                            type_name: type_str.to_string(),
                                            optional: false,
                                        });
                                    }
                                }
                            }

                            // Parse nested fields if they exist
                            if let Some(nested_section) = schema_def_table.get("nested_fields") {
                                if let Some(nested_table) = nested_section.as_table() {
                                    for (field_name, field_type) in nested_table {
                                        if let Some(type_str) = field_type.as_str() {
                                            nested_fields
                                                .insert(field_name.clone(), type_str.to_string());
                                        }
                                    }
                                }
                            }

                            (fields, nested_fields)
                        } else {
                            (Vec::new(), HashMap::new())
                        }
                    } else {
                        (Vec::new(), HashMap::new())
                    };

                    // Parse query parameters if they exist
                    let mut query_params = HashMap::new();
                    let query_key = format!("{}.query_params", name);
                    if let Some(query_section) = tables.get(&query_key) {
                        if let Some(query_table) = query_section.as_table() {
                            for (param_name, param_value) in query_table {
                                // Simple parameter definition with value
                                if let Some(value_str) = Self::toml_value_to_string(param_value) {
                                    let param_definition = QueryParamDefinition {
                                        name: param_name.clone(),
                                        param_type: Self::infer_param_type(&value_str),
                                        default: Some(Self::string_to_param_value(&value_str)),
                                        description: None,
                                    };
                                    query_params.insert(param_name.clone(), param_definition);
                                }
                            }
                        }
                    }

                    let schema = Schema {
                        name: name.clone(),
                        fields,
                        nested_fields,
                        query_params,
                    };

                    self.schemas.insert(name.clone(), schema);
                }
            }
        }

        Ok(())
    }

    /// Convert TOML value to string
    fn toml_value_to_string(value: &toml::Value) -> Option<String> {
        match value {
            toml::Value::String(s) => Some(s.clone()),
            toml::Value::Integer(i) => Some(i.to_string()),
            toml::Value::Float(f) => Some(f.to_string()),
            toml::Value::Boolean(b) => Some(b.to_string()),
            _ => None,
        }
    }

    /// Infer parameter type from string value
    fn infer_param_type(value: &str) -> String {
        if value.parse::<i64>().is_ok() {
            "i64".to_string()
        } else if value.parse::<f64>().is_ok() {
            "f64".to_string()
        } else if value.parse::<bool>().is_ok() {
            "bool".to_string()
        } else {
            "String".to_string()
        }
    }

    /// Convert string to parameter value
    fn string_to_param_value(value: &str) -> QueryParamValue {
        if let Ok(i) = value.parse::<i64>() {
            QueryParamValue::Integer(i)
        } else if let Ok(f) = value.parse::<f64>() {
            QueryParamValue::Float(f)
        } else if let Ok(b) = value.parse::<bool>() {
            QueryParamValue::Boolean(b)
        } else {
            QueryParamValue::String(value.to_string())
        }
    }

    /// Parse a query parameter definition from TOML (deprecated)
    #[allow(dead_code)]
    fn parse_query_param_definition(
        &self,
        name: &str,
        value: &toml::Value,
    ) -> Result<QueryParamDefinition, Box<dyn std::error::Error>> {
        if let Some(table) = value.as_table() {
            let param_type = table
                .get("type")
                .and_then(|v| v.as_str())
                .unwrap_or("String")
                .to_string();

            let default = if let Some(default_value) = table.get("default") {
                Some(match default_value {
                    toml::Value::String(s) => QueryParamValue::String(s.clone()),
                    toml::Value::Integer(i) => QueryParamValue::Integer(*i),
                    toml::Value::Float(f) => QueryParamValue::Float(*f),
                    toml::Value::Boolean(b) => QueryParamValue::Boolean(*b),
                    _ => QueryParamValue::String(default_value.to_string()),
                })
            } else {
                None
            };

            let description = table
                .get("description")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            Ok(QueryParamDefinition {
                name: name.to_string(),
                param_type,
                default,
                description,
            })
        } else if let Some(type_str) = value.as_str() {
            // Simple string format like "String" or "u32"
            Ok(QueryParamDefinition {
                name: name.to_string(),
                param_type: type_str.to_string(),
                default: None,
                description: None,
            })
        } else {
            Err(format!("Invalid query parameter definition for {}", name).into())
        }
    }

    /// Get a schema by name
    pub fn get_schema(&self, name: &str) -> Option<&Schema> {
        self.schemas.get(name)
    }

    /// List all available schema names
    pub fn list_schemas(&self) -> Vec<&String> {
        self.schemas.keys().collect()
    }

    /// Apply a schema to JSON data, returning a processed Value
    pub fn apply_schema(
        &self,
        schema_name: &str,
        data: &Value,
    ) -> Result<Value, Box<dyn std::error::Error>> {
        let _schema = self
            .get_schema(schema_name)
            .ok_or_else(|| format!("Schema '{}' not found", schema_name))?;

        // Handle paginated responses
        let data_to_process = if let Some(obj) = data.as_object() {
            // If this looks like a paginated response, process the "results" array
            if obj.contains_key("results") && obj.contains_key("count") {
                data
            } else {
                // For single objects, wrap them in a structure that matches our processing
                data
            }
        } else {
            data
        };

        if let Some(obj) = data_to_process.as_object() {
            let mut result = serde_json::Map::new();

            // Copy all fields by default
            for (key, value) in obj {
                result.insert(key.clone(), value.clone());
            }

            // Apply field-specific processing if needed
            // For now, we're just passing through the data as-is
            // In a more sophisticated implementation, you could:
            // 1. Validate field types
            // 2. Apply transformations
            // 3. Handle nested schema application

            Ok(Value::Object(result))
        } else {
            // For arrays or other types, pass through as-is
            Ok(data_to_process.clone())
        }
    }

    /// Build query parameters string from provided parameters
    pub fn build_query_string(
        &self,
        schema_name: &str,
        params: &HashMap<String, String>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let schema = self
            .get_schema(schema_name)
            .ok_or_else(|| format!("Schema '{}' not found", schema_name))?;

        let mut query_pairs = Vec::new();

        // Add provided parameters
        for (key, value) in params {
            if schema.query_params.contains_key(key) {
                query_pairs.push(format!("{}={}", key, urlencoding::encode(value)));
            }
        }

        // Add default parameters for any that weren't provided
        for (param_name, param_def) in &schema.query_params {
            if !params.contains_key(param_name) {
                if let Some(default_value) = &param_def.default {
                    query_pairs.push(format!(
                        "{}={}",
                        param_name,
                        urlencoding::encode(&default_value.to_string())
                    ));
                }
            }
        }

        if query_pairs.is_empty() {
            Ok(String::new())
        } else {
            Ok(format!("?{}", query_pairs.join("&")))
        }
    }
}

impl Default for SchemaManager {
    fn default() -> Self {
        Self::new()
    }
}
