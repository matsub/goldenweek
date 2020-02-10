mod message;

fn main() {
    match message::recv() {
        Ok(()) => {},
        Err(err) => println!("Error: {:?}", err),
    }
}
