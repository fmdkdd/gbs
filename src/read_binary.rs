use std::io::{self, Error, ErrorKind, Read};
use std::char;

pub fn read_u8(file: &mut Read) -> io::Result<u8> {
  let mut buf = [0; 1];
  try!(file.read(&mut buf));
  Ok(buf[0])
}

pub fn read_u16_le(file: &mut Read) -> io::Result<u16> {
  let mut buf = [0; 2];
  try!(file.read(&mut buf));
  Ok(((buf[1] as u16) << 8) | (buf[0] as u16))
}

pub fn read_all(file: &mut Read, buf: &mut [u8]) -> io::Result<()> {
  match file.read(buf) {
    Ok(len) if len == buf.len() => Ok(()),
    Ok(_) => Err(Error::new(ErrorKind::Other, "Could not read all bytes")),
    Err(e) => Err(e)
  }
}

pub fn read_str(file: &mut Read, len: u32) -> io::Result<String> {
  let mut ret = String::new();
  for _ in 0..len {
    let b = try!(read_u8(file));
    match char::from_u32(b as u32) {
      Some(c) if b != 0 => ret.push(c),
      _ => break
    };
  }
  Ok(ret)
}
