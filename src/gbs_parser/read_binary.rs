use std::io::{self, BufReader, Error, ErrorKind, Read};

// Provide the following methods on types implementing Read.
pub trait ReadBinary : Read {

  // Backported from unstable read_exact in 1.5.0 source.
  fn read_just(&mut self, mut buf: &mut [u8]) -> io::Result<()> {
    while !buf.is_empty() {
      match self.read(buf) {
        Ok(0) => break,
        Ok(n) => { let tmp = buf; buf = &mut tmp[n..]; }
        Err(ref e) if e.kind() == ErrorKind::Interrupted => {}
        Err(e) => return Err(e),
      }
    }
    if !buf.is_empty() {
      Err(Error::new(ErrorKind::Other,
                     "failed to fill whole buffer"))
    } else {
      Ok(())
    }
  }

  fn read_u8(&mut self) -> io::Result<u8> {
    let mut buf = [0];
    try!(self.read_just(&mut buf));
    Ok(buf[0])
  }

  fn read_u16_le(&mut self) -> io::Result<u16> {
    let mut buf = [0, 0];
    try!(self.read_just(&mut buf));
    Ok(((buf[1] as u16) << 8) | (buf[0] as u16))
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
