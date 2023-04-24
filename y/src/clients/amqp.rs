use serde::{Serialize, Deserialize};


#[derive(Serialize, Deserialize, Debug)]
pub struct PostMQ {
    pub title: String,
    pub body: String,
}

impl PostMQ {
    pub fn new(title: &str, body: &str) -> Self {
        PostMQ {
            title: title.to_string(),
            body: body.to_string(),
        }
    }
}
