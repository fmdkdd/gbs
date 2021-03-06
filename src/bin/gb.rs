extern crate gbs;

use std::env;

use gbs::gb_parser;
use gbs::screen;
use gbs::gb::{self, lcd};

#[macro_use]
extern crate glium;

// use glium::glutin::{Event, ElementState, VirtualKeyCode, MouseButton,
//                     MouseScrollDelta, TouchPhase};
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

  let mut gb = gb::GB::new();
  // Load
  gb.load_rom(&gbs.rom, 0);

  // Init screen
  let display = glium::glutin::WindowBuilder::new()
    .with_title("RustBoy")
    .with_dimensions((256 * SCREEN_ZOOM) as u32,
                     (256 * SCREEN_ZOOM) as u32)
    .build_glium().unwrap();
  let mut screen = screen::Screen::new(&display, 256, 256);

  let lcd = lcd::LCD::new();

  // Init
  gb.reset();

  // Play
  loop {
    gb.run_for(70224);

    let mut frame = display.draw();

    lcd.draw_tiles(&lcd.tiles(gb.tile_pattern_table()), &mut screen);
    screen.repaint(&mut frame);

    frame.finish().unwrap();
  }
}
