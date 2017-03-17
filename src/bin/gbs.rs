extern crate gbs;

use std::env;

use gbs::gbs_parser;
use gbs::gb::GB;
use gbs::gb::cpu::{R8, R16};

fn main() {
  let filename = env::args().nth(1)
    .expect("No GBS file specified");

  let gbs = gbs_parser::load(filename)
    .expect("Error loading GBS file");

  // println!("{:?}", gbs);
  println!("load_addr: {:x}", gbs.load_addr);
  println!("init_addr: {:x}", gbs.init_addr);
  println!("play_addr: {:x}", gbs.play_addr);
  println!("sp: {:x}", gbs.sp);

  println!("version: {}", gbs.version);
  println!("n_songs: {}", gbs.n_songs);
  println!("first_song: {}", gbs.first_song);
  println!("title: {}", gbs.title);
  println!("author: {}", gbs.author);
  println!("copyright: {}", gbs.copyright);
  println!("rom len: {:x}", gbs.rom.len());

  let mut gb = GB::new();
  // Load
  gb.load_rom(&gbs.rom, gbs.load_addr);

  // Init
  gb.cpu.clear_registers();
  gb.cpu.clear_ram();
  gb.cpu.rr_set(R16::SP, gbs.sp);
  gb.cpu.r_set(R8::A, gbs.first_song);
  gb.cpu.rr_set(R16::PC, gbs.init_addr);
  // Run INIT until RET
  while gb.cpu.read(gb.cpu.rr(R16::PC)) != 0xC9 {
    gb.cpu.step();
  }
  // Execute the RET
  gb.cpu.step();

  // Play
  gb.cpu.rr_set(R16::PC, gbs.play_addr);
  // FIXME: should run until RET, but the code can in fact contain CALL and
  // nested RET, so should run until the top RET (check SP?)
  // while gb.cpu.read(gb.cpu.rr(R16::PC)) != 0xC9 {
  //   gb.cpu.step();
  // }
  // Execute the RET
  // gb.cpu.step();
  gb.run();
}
