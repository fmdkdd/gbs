use std::io::{self, Error, ErrorKind, Read};
use std::fs::File;
use std::char;

// Provide the following methods on types implementing Read.
pub trait BinaryRead : Read {

  fn read_u8(&mut self) -> io::Result<u8> {
    let mut buf = [0];
    try!(self.read(&mut buf));
    Ok(buf[0])
  }

  fn read_u16_le(&mut self) -> io::Result<u16> {
    let mut buf = [0, 0];
    try!(self.read(&mut buf));
    Ok(((buf[1] as u16) << 8) | (buf[0] as u16))
  }

  fn read_all(&mut self, buf: &mut [u8]) -> io::Result<()> {
    match self.read(buf) {
      Ok(len) if len == buf.len() => Ok(()),
      Ok(_) => Err(Error::new(ErrorKind::Other, "Could not read all bytes")),
      Err(e) => Err(e)
    }
  }

  fn read_str(&mut self, len: u32) -> io::Result<String> {
    let mut ret = String::new();
    for _ in 0..len {
      let b = try!(self.read_u8());
      match char::from_u32(b as u32) {
        Some(c) if b != 0 => ret.push(c),
        _ => break
      };
    }
    Ok(ret)
  }
}

// We only need it on File for now.
impl BinaryRead for File {}
