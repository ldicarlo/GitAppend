use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub settings: String,
    pub appenders: HashMap<String, HashMap<String, Appender>>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Appender {
    pub source: String,
    // pub Option<sorted>: bool
    // pub Option<unique>: bool
    pub password: Option<String>,
}
