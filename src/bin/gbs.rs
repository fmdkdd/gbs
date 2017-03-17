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

  let mut samples = Vec::new();

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
    samples.push(gb.cpu.hardware.apu_step());
  }
  // Execute the RET
  gb.cpu.step();
  samples.push(gb.cpu.hardware.apu_step());

  // Play
  gb.cpu.rr_set(R16::PC, gbs.play_addr);
  // FIXME: should run until RET, but the code can in fact contain CALL and
  // nested RET, so should run until the top RET (check SP?)
  // while gb.cpu.read(gb.cpu.rr(R16::PC)) != 0xC9 {
  //   gb.cpu.step();
  // }
  // Execute the RET
  // gb.cpu.step();
  for _ in 0..20000 {
    gb.cpu.step();
    samples.push(gb.cpu.hardware.apu_step());
  }

  // Write to WAV
  let spec = hound::WavSpec {
    channels: 2,
    sample_rate: 44100,
    bits_per_sample: 16,
    sample_format: hound::SampleFormat::Int,
  };
  let max = std::i16::MAX as f32;
  let amp = 0.3;
  let mut writer = hound::WavWriter::create("out.wav", spec).unwrap();
  for s in samples {
    let sample = ((s as f32) / 30.0 - 1.0) * amp;
    writer.write_sample((sample * max) as i16).unwrap();
  }
}
