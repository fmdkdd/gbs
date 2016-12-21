extern crate gbs;

use std::env;
use std::io::{BufReader, Read};
use std::fs::File;

use gbs::gb_parser;
use gbs::screen;
use gbs::gb;

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

  let mut gb = gb::GB::new();

  // Load BIOS and ROM
  gb.load_rom(&gbs.rom, 0);

  let bios = File::open("boot.rom").expect("No BIOS rom found");
  let bios_file = BufReader::new(bios);
  let bios : Vec<u8> = bios_file.bytes().filter_map(|b| b.ok()).collect();
  gb.load_rom(&bios, 0);

  // Init screen
  let display = glium::glutin::WindowBuilder::new()
    .with_title("RustBoy")
    .with_dimensions((256 * SCREEN_ZOOM) as u32,
                     (256 * SCREEN_ZOOM) as u32)
    .build_glium().unwrap();
  let mut screen = screen::Screen::new(&display, 256, 256);

  let mut lcd = gb::lcd::LCD::new();

  // Reset
  gb.reset();

  // Play
  loop {
    gb.run_for(70224);

    let mut frame = display.draw();

    // Test the LCD functionality manually for now
    lcd.clear_background();

    // Should print full white screen with black dots at the corners

    // Tile 0: all white

    // Tile 1: black top-left corner
    // lcd.tile_data_1[16*1 + 8*0 + 0] = 0x80;
    // lcd.tile_data_1[16*1 + 8*0 + 1] = 0x80;

    // // Tile 2: black top-right corner
    // lcd.tile_data_1[16*2 + 8*0 + 0] = 0x01;
    // lcd.tile_data_1[16*2 + 8*0 + 1] = 0x01;

    // // Tile 3: black bottom-left corner
    // lcd.tile_data_1[16*3 + 2*7 + 0] = 0x80;
    // lcd.tile_data_1[16*3 + 2*7 + 1] = 0x80;

    // // Tile 4: black bottom-left corner
    // lcd.tile_data_1[16*4 + 2*7 + 0] = 0x01;
    // lcd.tile_data_1[16*4 + 2*7 + 1] = 0x01;

    // // Set the corners
    // lcd.bg_map_1[32*0  +  0] = 1;
    // lcd.bg_map_1[32*0  + 31] = 2;
    // lcd.bg_map_1[32*31 +  0] = 3;
    // lcd.bg_map_1[32*31 + 31] = 4;

    // Can also check scrolling
    // lcd.scroll_x = lcd.scroll_x.wrapping_add(1);
    // lcd.scroll_y = lcd.scroll_y.wrapping_add(1);
    lcd.draw_background(gb.tile_map(), gb.tile_data());
    screen.draw(|pixels| {
      for i in 0..lcd.pixels.len() {
        pixels[i] = lcd.pixels[i].as_intensity();
      }
    });


    // gb.cpu.lcd.draw_background();


    screen.repaint(&mut frame);

    frame.finish().unwrap();
  }
}
