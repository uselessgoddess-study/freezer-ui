use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Model {
    pub name: String,
    pub year: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Freezer {
    #[serde(rename = "_id")]
    pub name: String,

    pub model: Model,
    pub owner: Option<String>,
    pub products: HashMap<String, usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Product {
    #[serde(rename = "_id")]
    pub name: String,
    pub default: usize,
}
