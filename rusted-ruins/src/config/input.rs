use crate::game::Command;
use std::collections::HashMap;

#[derive(Deserialize, Debug)]
pub struct InputConfig {
    pub normal: HashMap<String, Command>,
    pub dialog: HashMap<String, Command>,
    pub targeting: HashMap<String, Command>,
}
