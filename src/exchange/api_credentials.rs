use serde::{Serialize, Deserialize};

use std::fs;

#[derive(Serialize, Deserialize, Clone)]
pub struct ApiCredentials {
    pub(crate) name: String,
    pub(crate) api_key: String,
    pub(crate) api_secret: String,
    pub(crate) api_pass: String,
}

pub fn load_api_credentials() -> Vec<ApiCredentials> {
    let file_str = fs::read_to_string("../../settings.json").expect("No settings.json file found!");
    serde_json::from_str::<Vec<ApiCredentials>>(&file_str).expect("Unable to deserialize settings.json!")
}
