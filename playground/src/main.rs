use rand::Rng;
use std::println;

//use questdb::{
//    ingress::{Buffer, SenderBuilder},
//    Result,
//};

fn main() -> Result<(), ()> {
    let x = "".to_string();

    if x.is_empty() {
        println!("is empty");
    }

    //rand::
    //let mut sender = SenderBuilder::new(host, port).connect()?;
    //let mut buffer = Buffer::new();
    //buffer
    //    .table("sensors")?
    //    .symbol("id", "toronto1")?
    //    .column_f64("temperature", 20.0)?
    //    .column_i64("humidity", 50)?
    //    .at_now()?;
    //sender.flush(&mut buffer)?;
    Ok(())
}
