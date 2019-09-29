use std::net::{Ipv4Addr, SocketAddrV4, UdpSocket};
use std::io;

fn recv()  -> Result<(), io::Error> {
    // Define the local connection information
    let ip = Ipv4Addr::new(0, 0, 0, 0);
    let connection = SocketAddrV4::new(ip, 3478);

    // Bind the socket
    let socket = UdpSocket::bind(connection)?;

    // Read from the socket
    let mut buf = [0; 1500];
    let (amt, _) = socket.recv_from(&mut buf)?;

    // Print only the valid data (slice)
    let mut i = 0;
    for b in &buf[0 .. amt] {
        print!("{:02x}", b);
        if i%2 == 1 {
            print!(" ");
        }
        i += 1;
    }

    println!("");

    Ok(())
}

fn main() {
    match recv() {
        Ok(()) => {},
        Err(err) => println!("Error: {:?}", err),
    }
}
