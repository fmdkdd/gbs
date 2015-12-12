use std::io::{self, Seek, SeekFrom};
use std::fs::File;
use std::path::Path;

mod read_binary;

const HEADER_LEN: usize = 3;
const HEADER_BYTES: &'static [u8; HEADER_LEN] = b"GBS";
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
  // rom:        [u8; ROM_LEN],
}

pub fn load<P: AsRef<Path>>(path: P) -> io::Result<Gbs> {
  let mut file = try!(File::open(path));

  let mut header = [0; HEADER_LEN];
  try!(read_binary::read_all(&mut file, &mut header));
  assert_eq!(header, *HEADER_BYTES);

  let version = try!(read_binary::read_u8(&mut file));
  let n_songs = try!(read_binary::read_u8(&mut file));
  let first_song = try!(read_binary::read_u8(&mut file));
  let load_addr = try!(read_binary::read_u16_le(&mut file));
  let init_addr = try!(read_binary::read_u16_le(&mut file));
  let play_addr = try!(read_binary::read_u16_le(&mut file));
  let sp = try!(read_binary::read_u16_le(&mut file));
  let timer_mod = try!(read_binary::read_u8(&mut file));
  let timer_ctrl = try!(read_binary::read_u8(&mut file));
  let title = try!(read_binary::read_str(&mut file, 32));
  try!(file.seek(SeekFrom::Start(0x30)));
  let author = try!(read_binary::read_str(&mut file, 32));
  try!(file.seek(SeekFrom::Start(0x50)));
  let copyright = try!(read_binary::read_str(&mut file, 32));
  // let mut rom = [0; ROM_LEN];
  // try!(read_all(&mut file, &mut rom));

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
    // rom:        rom,
  })
}
