use std::io::{self, BufReader, Read};

// Provide the following methods on types implementing Read.
pub trait ReadBinary : Read {

  fn read_u8(&mut self) -> io::Result<u8> {
    let mut buf = [0];
    try!(self.read_exact(&mut buf));
    Ok(buf[0])
  }

  fn read_u16_le(&mut self) -> io::Result<u16> {
    let mut buf = [0, 0];
    try!(self.read_exact(&mut buf));
    Ok(((buf[1] as u16) << 8) | (buf[0] as u16))
  }

  fn read_u16_be(&mut self) -> io::Result<u16> {
    let mut buf = [0, 0];
    try!(self.read_exact(&mut buf));
    Ok(((buf[0] as u16) << 8) | (buf[1] as u16))
  }

  fn read_str(&mut self, len: usize) -> io::Result<String> {
    let mut ret = String::new();
    for b in self.bytes().take(len) {
      let c = try!(b);
      if c != 0 {
        ret.push(c as char)
      }
    }
    Ok(ret)
  }
}

// We only need it on BufReader for now.
impl<R: Read> ReadBinary for BufReader<R> {}
