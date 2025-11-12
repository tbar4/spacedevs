use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Paginated<T> {
    pub count: u32,
    pub next: Option<String>,
    pub previous: Option<String>,
    pub results: Vec<T>,
}