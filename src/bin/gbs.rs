extern crate gbs;
extern crate hound;

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

  // Init WAV output
  let spec = hound::WavSpec {
    channels: 1,
    sample_rate: 44100,
    bits_per_sample: 16,
    sample_format: hound::SampleFormat::Int,
  };
  let max = 0.3 * (std::i16::MAX as f32);
  let mut writer = hound::WavWriter::create("out.wav", spec).unwrap();

  let idle_addr = 0xF00D;
  gb.cpu.rst_offset = gbs.load_addr;

  // Load
  gb.load_rom(&gbs.rom, gbs.load_addr);

  // Init
  gb.cpu.clear_registers();
  gb.cpu.clear_ram();

  gb.cpu.write(idle_addr, 0xFF);

  gb.cpu.rr_set(R16::SP, gbs.sp);
  gb.cpu.r_set(R8::A, 0);
  gb.cpu.rr_set(R16::PC, idle_addr);
  gb.cpu.call(gbs.init_addr);
  // Run the INIT subroutine
  while gb.cpu.rr(R16::PC) != idle_addr {
    let cycles = gb.cpu.step();
  }
  // Run the instruction at idle_addr for debug log
  gb.cpu.step();

  // Play
  gb.cpu.rr_set(R16::PC, gbs.play_addr);
  let mut elapsed = 0;
  while elapsed < 4194304 {
    let cycles = gb.cpu.step();
    for _ in 0..cycles {
      gb.cpu.hardware.apu_step();

      // Downsample
      if elapsed % 95 == 0 {
        writer.write_sample((gb.cpu.hardware.apu_output() * max) as i16).unwrap();
      }
    }
    elapsed += cycles as u64;
  }
}
