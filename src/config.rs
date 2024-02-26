use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub settings: String,
    pub appenders: HashMap<String, Appender>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Appender {
    pub git_folder_location: String,
    // pub Option<sorted>: bool
    // pub Option<unique>: bool
}
