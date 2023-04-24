use serde::{Serialize, Deserialize};
use clients::amqp::PostMQ;

pub mod clients;

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

    pub fn print(&self) {
        let x = PostMQ::new("ca", "pi");
        print!("{}", x.title);
    }
}