#![feature(non_exhaustive)]
extern crate core;
extern crate scroll;
use scroll::{Cwrite, Pread};

#[path = "../../src/protocol.rs"]
mod protocol;
use protocol::MsgType;

#[test]
fn msgtype_round_trip() {
    let mut buffer: [u8; 5] = [255; 5];
    let e = MsgType::Error;
    let f = MsgType::FwInfo;
    buffer.cwrite(e, 0);
    buffer.cwrite(f, 4);
    assert_eq!(e, buffer.pread(0).unwrap());
    assert_eq!(f, buffer.pread(4).unwrap());
    assert_eq!(buffer, [1, 255, 255, 255, 10]);
    assert!(buffer.pread::<MsgType>(2).is_err());
}

fn main() {}
