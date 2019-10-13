mod stun;

fn main() {
    match stun::recv() {
        Ok(()) => {},
        Err(err) => println!("Error: {:?}", err),
    }
}
