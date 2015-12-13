extern crate gbs;

use std::env;

use gbs::gbs_parser;
use gbs::cpu;

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
  // println!("first_song: {}", gbs.first_song);
  println!("title: {}", gbs.title);
  println!("author: {}", gbs.author);
  println!("copyright: {}", gbs.copyright);
  println!("rom len: {:x}", gbs.rom.len());

  let mut cpu = cpu::Cpu::new();
  // Load
  cpu.load_rom(&gbs.rom, gbs.load_addr as usize);

  // Init
  cpu.reset();
  cpu.a = gbs.first_song;
  cpu.pc = gbs.init_addr;

  // Play
  cpu.run(10);
}
