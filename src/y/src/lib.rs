use serde::{Deserialize, Serialize};

pub mod clients;
pub mod utils;

#[derive(Serialize, Deserialize, Debug)]
pub struct Post {
    pub title: String,
    body: String,
}

impl Post {
    pub fn new(title: &str, body: &str) -> Post {
        Post {
            title: title.to_string(),
            body: body.to_string(),
        }
    }
}
