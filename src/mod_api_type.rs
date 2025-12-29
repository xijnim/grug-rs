use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct ModAPI {
    pub entities: HashMap<String, Entity>,
    pub game_functions: HashMap<String, GameFunction>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Entity {
    pub description: String,
    pub on_functions: HashMap<String, OnFunction>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OnFunction {
    pub description: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GameFunction {
    pub description: String,
    #[serde(default)]
    pub arguments: Vec<Argument>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Argument {
    pub name: String,
    #[serde(rename(serialize = "type", deserialize = "type"))]
    pub type_: String,
}
