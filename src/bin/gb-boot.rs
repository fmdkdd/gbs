extern crate gbs;

use std::env;
use std::io::{BufReader, Read};
use std::fs::File;

use gbs::gb_parser;
use gbs::cpu;
use gbs::screen::{self, Screen};

#[macro_use]
extern crate glium;

use glium::DisplayBuild;

const SCREEN_ZOOM: usize = 4;

fn main() {
  let filename = env::args().nth(1)
    .expect("No GB file specified");

  let gbs = gb_parser::load(filename)
    .expect("Error loading GB file");

  println!("Title: {}", gbs.title);
  println!("Cartridge type: {}", gbs.cartridge_type);
  println!("ROM size: {}", gbs.rom_size);

  let mut cpu = cpu::Cpu::new();

  // Load BIOS and ROM
  cpu.load_rom(&gbs.rom, 0);

  let bios = File::open("boot.rom")
    .expect("No BIOS rom found");
  let bios_file = BufReader::new(bios);
  let bios : Vec<u8> = bios_file.bytes().filter_map(|b| b.ok()).collect();
  cpu.load_rom(&bios, 0);

  // Init screen
  let display = glium::glutin::WindowBuilder::new()
    .with_title("GBS")
    .with_dimensions((screen::SCREEN_WIDTH * SCREEN_ZOOM) as u32,
                     (screen::SCREEN_HEIGHT * SCREEN_ZOOM) as u32)
    .build_glium().unwrap();
  let mut screen = Screen::new(&display);

  // Init
  cpu.reset();

  // Play
  loop {
    cpu.run_for(70224);

    let mut frame = display.draw();

    screen.draw_tile_table(cpu.tile_table());
    screen.repaint(&mut frame);

    frame.finish().unwrap();
  }
}
