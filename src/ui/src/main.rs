use askama::Template;

#[derive(Template)]
#[template(path = "hello.html")]

struct HelloTemplate<'a> {
    name: &'a str,
}

fn main() {
    let hello = HelloTemplate { name: "world" };
    println!("{}", hello.render().unwrap());
}
