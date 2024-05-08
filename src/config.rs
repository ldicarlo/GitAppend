use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub settings: String,
    pub appenders: HashMap<String, GitAppender>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct GitAppender {
    pub git_config: Option<GitConfig>,
    pub links: HashMap<String, Appender>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Appender {
    pub source: String,
    // pub Option<sorted>: bool
    // pub Option<unique>: bool
    pub password_file: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct GitConfig {
    pub username: String,
    pub token_file: String,
}
