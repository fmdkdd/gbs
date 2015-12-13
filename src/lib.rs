use std::io::{self, BufReader, Error, ErrorKind, Read};
use std::fs::File;
use std::path::Path;

mod read_binary;

use read_binary::BinaryRead;

// const ROM_LEN: usize = 0x8000;

#[derive(Debug)]
pub struct Gbs {
  pub version:    u8,
  pub n_songs:    u8,
  pub first_song: u8,
  pub load_addr:  u16,
  pub init_addr:  u16,
  pub play_addr:  u16,
  pub sp:         u16,
  pub timer_mod:  u8,
  pub timer_ctrl: u8,
  pub title:      String,
  pub author:     String,
  pub copyright:  String,
  pub rom:        Vec<u8>,
}

pub fn load<P: AsRef<Path>>(path: P) -> io::Result<Gbs> {
  let f = try!(File::open(path));
  let mut file = BufReader::new(f);

  let header = try!(file.read_str(3));
  if header != "GBS" {
    return Err(Error::new(ErrorKind::Other,
                          "Wrong header, not a GBS file"))
  }

  let version = try!(file.read_u8());
  let n_songs = try!(file.read_u8());
  let first_song = try!(file.read_u8());
  let load_addr = try!(file.read_u16_le());
  let init_addr = try!(file.read_u16_le());
  let play_addr = try!(file.read_u16_le());
  let sp = try!(file.read_u16_le());
  let timer_mod = try!(file.read_u8());
  let timer_ctrl = try!(file.read_u8());
  let title = try!(file.read_str(32));
  let author = try!(file.read_str(32));
  let copyright = try!(file.read_str(32));
  let rom : Vec<u8> = try!(file.bytes().collect());

  Ok(Gbs {
    version:    version,
    n_songs:    n_songs,
    first_song: first_song,
    load_addr:  load_addr,
    init_addr:  init_addr,
    play_addr:  play_addr,
    sp:         sp,
    timer_mod:  timer_mod,
    timer_ctrl: timer_ctrl,
    title:      title,
    author:     author,
    copyright:  copyright,
    rom:        rom,
  })
}
