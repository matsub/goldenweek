use std::net::{Ipv4Addr, SocketAddrV4, UdpSocket};
use std::thread;
use std::io;

pub fn recv()  -> Result<(), io::Error> {
    // Define the local connection information
    let ip = Ipv4Addr::new(0, 0, 0, 0);
    let connection = SocketAddrV4::new(ip, 3478);

    // Bind the socket
    let socket = UdpSocket::bind(connection)?;

    loop {
        // Read from the socket
        let mut buf = [0; 1500];
        let s = socket.try_clone()?;
        match  socket.recv_from(&mut buf) {
            Ok((amt, src)) => {
                thread::spawn(move || {
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
                    println!("src: {}?", src);

                    let send_buf: [u8; 96] = [
                        0x01, 0x01, 0x00, 0x4c, 0x21, 0x12, 0xa4, 0x42, 0x54, 0xa6, 0xbb, 0xb1, 0x79, 0x14, 0x7f, 0x4c,
                        0x5d, 0x12, 0x97, 0xa1, 0x00, 0x20, 0x00, 0x08, 0x00, 0x01, 0xf2, 0x8b, 0x8d, 0x06, 0xa4, 0x41,
                        0x00, 0x01, 0x00, 0x08, 0x00, 0x01, 0xd3, 0x99, 0xac, 0x14, 0x00, 0x03, 0x80, 0x2b, 0x00, 0x08,
                        0x00, 0x01, 0x0d, 0x96, 0xac, 0x14, 0x00, 0x02, 0x80, 0x2c, 0x00, 0x08, 0x00, 0x01, 0x0d, 0x97,
                        0xac, 0x14, 0x00, 0x02, 0x80, 0x22, 0x00, 0x16, 0x43, 0x6f, 0x74, 0x75, 0x72, 0x6e, 0x2d, 0x34,
                        0x2e, 0x32, 0x2e, 0x31, 0x2e, 0x32, 0x20, 0x27, 0x4d, 0x6f, 0x6e, 0x7a, 0x61, 0x27, 0x00, 0x00,
                    ];

                    s.send_to(&send_buf, src).expect("failed to send response");;
                });
            },
            Err(e) => {
                panic!(e);
            }
        }
    }
}
