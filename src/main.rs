use std::io::{Result, Error, ErrorKind, Read, Seek, SeekFrom};
use std::char;
use std::env;
use std::fs::File;
use std::path::Path;

macro_rules! fail {
  ($expr:expr) => (return Err(Error::new(ErrorKind::Other, $expr)))
}

fn read_u8(file: &mut Read) -> Result<u8> {
  let mut buf = [0; 1];
  try!(file.read(&mut buf));
  Ok(buf[0])
}

fn read_u16_le(file: &mut Read) -> Result<u16> {
  let mut buf = [0; 2];
  try!(file.read(&mut buf));
  Ok(((buf[1] as u16) << 8) | (buf[0] as u16))
}

fn read_all(file: &mut Read, buf: &mut [u8]) -> Result<()> {
  match file.read(buf) {
    Ok(len) if len == buf.len() => Ok(()),
    Ok(_) => Err(Error::new(ErrorKind::Other, "Could not read all bytes")),
    Err(e) => Err(e)
  }
}

fn read_str(file: &mut Read, len: u32) -> Result<String> {
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

const HEADER_LEN: usize = 3;
const HEADER_BYTES: &'static [u8; HEADER_LEN] = b"GBS";
// const ROM_LEN: usize = 0x8000;

#[derive(Debug)]
struct Gbs {
  version:    u8,
  n_songs:    u8,
  first_song: u8,
  load_addr:  u16,
  init_addr:  u16,
  play_addr:  u16,
  sp:         u16,
  timer_mod:  u8,
  timer_ctrl: u8,
  title:      String,
  author:     String,
  copyright:  String,
  // rom:        [u8; ROM_LEN],
}

fn load<P: AsRef<Path>>(path: P) -> Result<Gbs> {
  let mut file = try!(File::open(path));

  let mut header = [0; HEADER_LEN];
  try!(read_all(&mut file, &mut header));
  if &header != HEADER_BYTES {
    fail!("Invalid header");
  }

  let version = try!(read_u8(&mut file));
  let n_songs = try!(read_u8(&mut file));
  let first_song = try!(read_u8(&mut file));
  let load_addr = try!(read_u16_le(&mut file));
  let init_addr = try!(read_u16_le(&mut file));
  let play_addr = try!(read_u16_le(&mut file));
  let sp = try!(read_u16_le(&mut file));
  let timer_mod = try!(read_u8(&mut file));
  let timer_ctrl = try!(read_u8(&mut file));
  let title = try!(read_str(&mut file, 32));
  try!(file.seek(SeekFrom::Start(0x30)));
  let author = try!(read_str(&mut file, 32));
  try!(file.seek(SeekFrom::Start(0x50)));
  let copyright = try!(read_str(&mut file, 32));
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

fn main() {
  let filename = match env::args().nth(1) {
    Some(s) => s,
    _ => panic!("No GBS file specified")
  };
  let gbs = match load(filename) {
    Ok(x) => x,
    Err(e) => panic!(e.to_string())
  };

  println!("{:?}", gbs);
  println!("load_addr: {:x}", gbs.load_addr);
  println!("init_addr: {:x}", gbs.init_addr);
  println!("play_addr: {:x}", gbs.play_addr);
  println!("sp: {:x}", gbs.sp);

  // println!("version: {}", gbs.version);
  // println!("n_songs: {}", gbs.n_songs);
  // println!("first_song: {}", gbs.first_song);
  // println!("title: {}", gbs.title);
  // println!("author: {}", gbs.author);
  // println!("copyright: {}", gbs.copyright);
}
