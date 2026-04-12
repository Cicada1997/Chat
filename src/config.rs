use std::fs;

use crate::Error;

use {
    serde::{
        Serialize,
        Deserialize,
    },
};

#[derive(Serialize, Deserialize)]
pub struct Auth {
    pub url: String,
    pub required: bool,
}

#[derive(Serialize, Deserialize)]
pub struct Chat {
    pub ip:   String,
    pub port: Option<u16>,
    
    pub database_url: String,
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub chat: Chat,
    pub auth: Auth,
}

impl Default for Config {
    fn default() -> Self {
        Self { 
            chat: Chat {
                ip: String::from("127.0.0.1"),
                port: Some(1997),
                database_url: String::from("db.kattmys.se"),
            },
            auth: Auth {
                url: String::from("auth.kattmys.se"),
                required: true,
            },
        }
    }
}

pub fn conf() -> Result<Config, Error> {
    // default if error when opening file or parsing to toml
    let conf = match fs::read_to_string("config.toml") {
        Ok(f) => toml::from_str::<Config>(&f).unwrap_or_default(),
        Err(_) => Config::default(),
    };

    Ok(conf)
}

