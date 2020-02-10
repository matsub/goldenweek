use std::net::{Ipv4Addr, SocketAddrV4, UdpSocket};
use byteorder::{BigEndian, WriteBytesExt, ReadBytesExt};
use std::thread;
use std::io;
use std::io::Cursor;

mod attributes;


#[allow(dead_code)]
enum ResponseType {
    Request,
    Indication,
    SuccessResponse,
    ErrorResponse,
}


#[allow(dead_code)]
struct Message<'a> {
    pub header: Header<'a>,
    pub attributes: Vec<Attribute>,
}


struct Header<'a> {
    pub message_type: u16,
    pub message_length: u16,
    pub magic_cookie: u32,
    pub transaction_id: &'a [u8],
}


impl Header<'_> {
    fn new(message_type: ResponseType, message_length: u16, transaction_id: &[u8]) -> Header {
        // only support binding request
        let message_type_value: u16 = match message_type {
           ResponseType::Request => 0x0001,
           ResponseType::Indication => 0x0011,
           ResponseType::SuccessResponse => 0x0101,
           ResponseType::ErrorResponse => 0x0111,
        };

        return Header {
            message_type: message_type_value,
            message_length: message_length,
            magic_cookie: 0x2112a442,
            transaction_id: transaction_id,
        };
    }

    fn parse(recv_buf: &[u8]) -> Result<Header, io::Error> {
        let mut rdr = Cursor::new(recv_buf);

        Ok(Header {
            message_type: rdr.read_u16::<BigEndian>()?,
            message_length: rdr.read_u16::<BigEndian>()?,
            magic_cookie: 0x2112a442,
            transaction_id: &recv_buf[8..20],
        })
    }

    fn packetize(&self) -> Result<Vec<u8>, io::Error> {
        let mut header = vec![];

        header.write_u16::<BigEndian>(self.message_type)?;
        header.write_u16::<BigEndian>(self.message_length)?;
        header.write_u32::<BigEndian>(self.magic_cookie)?;
        for &b in self.transaction_id {
            header.write_u8(b)?;
        }

        Ok(header)
    }
}


#[allow(dead_code)]
struct Attribute {
    pub attribute_type: u16,
    pub attribute_length: u16,
    pub value: Vec<u8>,
}


// header validation
pub fn validate_header(buf: &[u8]) -> bool {
    // Bindingメソッドのみサポートするよ
    // check the first two bits are 0
    if buf[0] & 0xc0 != 0 {
        return false;
    }

    // the magic cookie field has the correct value
    if &buf[4..8] != [0x21, 0x12, 0xa4, 0x42] {
        return false;
    }

    // check message class
    // request
    if ( buf[0] & 0x01 == 0) && ( buf[1] & 0x10 == 0) {
        return true;
    }

    // indication
    if ( buf[0] & 0x01 == 0) && ( buf[1] & 0x10 == 1) {
        return false;
    }

    // success response
    if ( buf[0] & 0x01 == 1) && ( buf[1] & 0x10 == 0) {
        return false;
    }

    // error response
    if ( buf[0] & 0x01 == 1) && ( buf[1] & 0x10 == 1) {
        return false;
    }

    // transaction ID
    // &buf[8..24]

    return true;
}


fn generate_response_message() -> Result<Vec<u8>, io::Error> {
    let mut message = vec![];

    // 0x0020: XOR-MAPPED-ADDRESS
    {
        let header: u16 = 0x0020;
        let length: u16 = 0x0008;

        let family: u16 = 0x0001;
        let x_port: u16 = 0xf28b;
        let x_address: [u8; 4] = [0x8d, 0x06, 0xa4, 0x41];

        message.write_u16::<BigEndian>(header)?;
        message.write_u16::<BigEndian>(length)?;
        message.write_u16::<BigEndian>(family)?;
        message.write_u16::<BigEndian>(x_port)?;
        for &b in &x_address {
            message.write_u8(b)?;
        }
    }

    // 0x0001: MAPPED-ADDRESS
    {
        let header: u16 = 0x0001;
        let length: u16 = 0x0008;

        let family: u16 = 0x0001;
        let port: u16 = 0xd399;
        let address: [u8; 4] = [0xac, 0x14, 0x00, 0x03];

        message.write_u16::<BigEndian>(header)?;
        message.write_u16::<BigEndian>(length)?;
        message.write_u16::<BigEndian>(family)?;
        message.write_u16::<BigEndian>(port)?;
        for &b in &address {
            message.write_u8(b)?;
        }
    }

    // 0x802b: RESPONSE-ORIGIN
    {
        let header: u16 = 0x802b;
        let length: u16 = 0x0008;

        let family: u16 = 0x0001;
        let port: u16 = 0x0d96;
        let address: [u8; 4] = [0xac, 0x14, 0x00, 0x02];

        message.write_u16::<BigEndian>(header)?;
        message.write_u16::<BigEndian>(length)?;
        message.write_u16::<BigEndian>(family)?;
        message.write_u16::<BigEndian>(port)?;
        for &b in &address {
            message.write_u8(b)?;
        }
    }

    // 0x802c: OTHER-ADDRESS
    {
        let header: u16 = 0x802c;
        let length: u16 = 0x0008;

        let family: u16 = 0x0001;
        let x_port: u16 = 0x0d97;
        let address: [u8; 4] = [0xac, 0x14, 0x00, 0x02];

        message.write_u16::<BigEndian>(header)?;
        message.write_u16::<BigEndian>(length)?;
        message.write_u16::<BigEndian>(family)?;
        message.write_u16::<BigEndian>(x_port)?;
        for &b in &address {
            message.write_u8(b)?;
        }
    }

    // 0x8022: SOFTWARE
    {
        let header: u16 = 0x8022;
        let length: u16 = 0x0016;

        let body: [u8; 0x16] = [
            0x43, 0x6f, 0x74, 0x75, 0x72,
            0x6e, 0x2d, 0x34, 0x2e, 0x32,
            0x2e, 0x31, 0x2e, 0x32, 0x20,
            0x27, 0x4d, 0x6f, 0x6e, 0x7a,
            0x61, 0x27
        ];

        message.write_u16::<BigEndian>(header)?;
        message.write_u16::<BigEndian>(length)?;
        for &b in &body {
            message.write_u8(b)?;
        }
    }

    for _ in 0..(message.len() % 4) {
        message.write_u8(0x00)?;
    }

    Ok(message)
}


fn generate_response(req_header: &Header) -> Result<Vec<u8>, io::Error> {
    let mut send_buf = vec![];

    let mut message = generate_response_message()?;
    let header = Header::new(
        ResponseType::SuccessResponse,
        message.len() as u16,
        req_header.transaction_id
        );

    send_buf.append(&mut header.packetize()?);
    send_buf.append(&mut message);

    Ok(send_buf)
}


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

                    let hdr = Header::parse(&buf[..]).unwrap();
                    println!("");
                    println!("src: {}", src);
                    println!("type: {:x}", hdr.message_type);
                    println!("length: {:x}", hdr.message_length);
                    println!("cookie: {:x}", hdr.magic_cookie);
                    println!("trans_id: {:x?}", hdr.transaction_id);

                    let send_buf = generate_response(&hdr).unwrap();

                    if validate_header(&buf) {
                        s.send_to(&send_buf, src).expect("failed to send response");
                    }
                });
            },
            Err(e) => {
                panic!(e);
            }
        }
    }
}
