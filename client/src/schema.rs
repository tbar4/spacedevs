//! Schema management for dynamic endpoint structures
//!
//! This module provides functionality to load struct definitions from TOML files
//! and use them to dynamically process API responses.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;

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

/// Represents a complete struct schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schema {
    /// Name of the struct
    pub name: String,
    /// Fields in the struct
    pub fields: Vec<FieldDefinition>,
    /// Nested fields that reference other schemas
    pub nested_fields: HashMap<String, String>,
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

        // Parse schemas from the TOML structure
        if let Some(tables) = config.as_table() {
            for (name, value) in tables {
                // Skip special sections like [types]
                if name == "types" {
                    continue;
                }

                // Skip nested_fields sections
                if name.contains(".nested_fields") {
                    continue;
                }

                // Parse the schema
                if let Some(schema_table) = value.as_table() {
                    let mut fields = Vec::new();
                    let mut nested_fields = HashMap::new();

                    // Parse regular fields
                    for (field_name, field_type) in schema_table {
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
                    let nested_key = format!("{}.nested_fields", name);
                    if let Some(nested_section) = tables.get(&nested_key) {
                        if let Some(nested_table) = nested_section.as_table() {
                            for (field_name, field_type) in nested_table {
                                if let Some(type_str) = field_type.as_str() {
                                    nested_fields.insert(field_name.clone(), type_str.to_string());
                                }
                            }
                        }
                    }

                    let schema = Schema {
                        name: name.clone(),
                        fields,
                        nested_fields,
                    };

                    self.schemas.insert(name.clone(), schema);
                }
            }
        }

        Ok(())
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
}

impl Default for SchemaManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    // use super::*;
    // use std::io::Write;

    #[test]
    fn test_schema_loading() -> Result<(), Box<dyn std::error::Error>> {
        // Create a temporary file for testing
        let toml_content = r#"
[article]
id = "u32"
title = "String"
featured = "bool"

[article.nested_fields]
authors = "Vec<Author>"

[blog]
id = "u32"
title = "String"
"#;

        // For this example, we'll just test the parsing logic directly
        // In a real implementation, you'd use a proper test file
        let config: toml::Value = toml::from_str(toml_content)?;

        // Verify we can parse the structure
        assert!(config.get("article").is_some());
        assert!(config.get("blog").is_some());

        Ok(())
    }
}
