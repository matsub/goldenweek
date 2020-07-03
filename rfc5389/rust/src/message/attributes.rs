// # STUN Attribute Registry from rfc5389
// Comprehension-required range (0x0000-0x7FFF):
//     0x0000: (Reserved)
//     0x0001: MAPPED-ADDRESS
//     0x0002: (Reserved; was RESPONSE-ADDRESS)
//     0x0003: (Reserved; was CHANGE-ADDRESS)
//     0x0004: (Reserved; was SOURCE-ADDRESS)
//     0x0005: (Reserved; was CHANGED-ADDRESS)
//     0x0006: USERNAME
//     0x0007: (Reserved; was PASSWORD)
//     0x0008: MESSAGE-INTEGRITY
//     0x0009: ERROR-CODE
//     0x000A: UNKNOWN-ATTRIBUTES
//     0x000B: (Reserved; was REFLECTED-FROM)
//     0x0014: REALM
//     0x0015: NONCE
//     0x0020: XOR-MAPPED-ADDRESS
//
//   Comprehension-optional range (0x8000-0xFFFF)
//     0x8022: SOFTWARE
//     0x8023: ALTERNATE-SERVER
//     0x8028: FINGERPRINT



#[allow(dead_code)]
pub enum AttributeKind {
    MappedAddress,
    Username,
    MessageIntegrity,
    ErrorCode,
    UnknownAttributes,
    Realm,
    Nonce,
    XorMappedAddress,
    Software,
    AlternateServer,
    Fingerprint,
}


#[allow(dead_code)]
trait Attribute {
    // convert packet buffer to an Attribute instance
    fn parse(buf: &[u8]) -> Result<Self, io::Error>;

    // generate packet buffer
    fn packetize(&self) -> Result<Vec<u8>, io::Error>;
}


#[allow(dead_code)]
struct MappedAddress {
}


#[allow(dead_code)]
struct Username {
}


#[allow(dead_code)]
struct MessageIntegrity {
}


#[allow(dead_code)]
struct ErrorCode {
}


#[allow(dead_code)]
struct UnknownAttributes {
}


#[allow(dead_code)]
struct Realm {
}


#[allow(dead_code)]
struct Nonce {
}


#[allow(dead_code)]
struct XorMappedAddress {
}


#[allow(dead_code)]
struct Software {
}


#[allow(dead_code)]
struct AlternateServer {
}


#[allow(dead_code)]
struct Fingerprint {
}
