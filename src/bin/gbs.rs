extern crate gbs;
extern crate hound;

use std::env;

use gbs::gbs_parser;
use gbs::gb::GB;
use gbs::gb::cpu::{R8, R16};

fn main() {
  // Parse args
  let filename = env::args().nth(1)
    .expect("No GBS file specified");

  let track = env::args().nth(2).map(|s| s.parse::<u8>())
    .unwrap_or(Ok(0)).unwrap_or(0);

  // Read GBS file
  let gbs = gbs_parser::load(filename)
    .expect("Error loading GBS file");

  println!("load_addr: {:x}", gbs.load_addr);
  println!("init_addr: {:x}", gbs.init_addr);
  println!("play_addr: {:x}", gbs.play_addr);
  println!("sp: {:x}", gbs.sp);
  println!("timer mod: {:x}", gbs.timer_mod);
  println!("timer control: {:x}", gbs.timer_ctrl);

  println!("version: {}", gbs.version);
  println!("n_songs: {}", gbs.n_songs);
  println!("first_song: {}", gbs.first_song);
  println!("title: {}", gbs.title);
  println!("author: {}", gbs.author);
  println!("copyright: {}", gbs.copyright);
  println!("rom len: {:x}", gbs.rom.len());

  // Bail if track doesn't exist
  if track >= gbs.n_songs {
    panic!("Requested track {} but only {} are available");
  } else {
    println!("Writing 1min of track {} to out.wav...", track);
  }

  // Init WAV output
  let spec = hound::WavSpec {
    channels: 2,
    sample_rate: 44100,
    bits_per_sample: 16,
    sample_format: hound::SampleFormat::Int,
  };
  let max = std::i16::MAX as f32;
  let mut writer = hound::WavWriter::create("out.wav", spec).unwrap();

  // Init emu
  let mut gb = GB::new();
  let idle_addr = 0xF00D;
  gb.cpu.rst_offset = gbs.load_addr;

  // Load
  gb.load_rom(&gbs.rom, gbs.load_addr);

  // Init
  gb.cpu.clear_registers();
  gb.cpu.clear_ram();

  gb.cpu.rr_set(R16::SP, gbs.sp);
  gb.cpu.r_set(R8::A, track);
  gb.cpu.rr_set(R16::PC, idle_addr);
  gb.cpu.call(gbs.init_addr);
  // Run the INIT subroutine
  while gb.cpu.rr(R16::PC) != idle_addr {
    gb.cpu.step();
  }

  // Play for 1min
  let mut frames = 60 * 60;
  let mut cycle = 0;
  while frames > 0 {
    // Emulate from play_addr at 60Hz
    let mut frame_period = 70224u32;
    gb.cpu.call(gbs.play_addr);

    // Run until PLAY has finished
    while gb.cpu.rr(R16::PC) != idle_addr {
      let cycles = gb.cpu.step();
      for _ in 0..cycles {
        gb.cpu.hardware.apu_step();

        // Downsample
        if cycle % 95 == 0 {
          let (left, right) = gb.cpu.hardware.apu_output();
          writer.write_sample((left * max) as i16).unwrap();
          writer.write_sample((right * max) as i16).unwrap();
        }
        cycle += 1;
      }
      frame_period -= cycles as u32;
    }

    // PLAY has finished for this frame, but we still need to run the APU until
    // the next frame
    for _ in 0..frame_period {
      gb.cpu.hardware.apu_step();

      // Downsample
      if cycle % 95 == 0 {
        let (left, right) = gb.cpu.hardware.apu_output();
        writer.write_sample((left * max) as i16).unwrap();
        writer.write_sample((right * max) as i16).unwrap();
      }
      cycle += 1;
    }
    frames -= 1;
  }

  println!("Done");
}
