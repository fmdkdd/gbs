use std::io::{self, BufReader, Read};
use std::fs::File;
use std::path::Path;
use std::result;

mod read_binary;

use gbs_parser::read_binary::ReadBinary;

macro_rules! fail {
  ($err:path) => (return Err($err));
}

pub type Result<T> = result::Result<T, GbsError>;

#[derive(Debug)]
pub enum GbsError {
  Io(io::Error),
  WrongHeader,
  UnknownVersion,
  InvalidAddress,
}

impl From<io::Error> for GbsError {
  fn from(err: io::Error) -> GbsError {
    GbsError::Io(err)
  }
}

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

pub fn load<P: AsRef<Path>>(path: P) -> self::Result<Gbs> {
  let f = try!(File::open(path));
  let mut file = BufReader::new(f);

  let header = try!(file.read_str(3));
  if header != "GBS" {
    fail!(GbsError::WrongHeader)
  }

  let version = try!(file.read_u8());
  if version != 1 {
    fail!(GbsError::UnknownVersion)
  }

  let n_songs = try!(file.read_u8());
  let first_song = try!(file.read_u8());

  let load_addr = try!(file.read_u16_le());
  if load_addr < 0x400 || load_addr > 0x7fff {
    fail!(GbsError::InvalidAddress)
  }

  let init_addr = try!(file.read_u16_le());
  if init_addr < 0x400 || init_addr > 0x7fff {
    fail!(GbsError::InvalidAddress)
  }

  let play_addr = try!(file.read_u16_le());
  if play_addr < 0x400 || play_addr > 0x7fff {
    fail!(GbsError::InvalidAddress)
  }

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
