use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Deserialize, Serialize, Clone, Eq, PartialEq, Hash)]
pub enum Feature {
    RemoveMultilinesBash,
}

#[derive(Debug, Deserialize, Serialize, Clone, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub appenders: HashMap<String, GitAppender>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct GitAppender {
    pub git_config: Option<GitConfig>,
    #[serde(default = "HashMap::new")]
    pub links: HashMap<String, GitLink>,
    #[serde(default = "HashMap::new")]
    pub folder_links: HashMap<String, GitLink>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct GitLink {
    pub source_path: String,
    pub source_branch: Option<String>,
    // pub Option<sorted>: bool
    // pub Option<unique>: bool
    pub password_file: Option<String>,
    pub remove_lines: Option<HashSet<String>>,
    pub exclude_patterns: Option<HashSet<String>>,
    pub features: Option<HashSet<Feature>>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct GitConfig {
    pub username: String,
    pub token_file: String,
}
