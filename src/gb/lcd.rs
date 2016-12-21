use super::super::screen::Screen;

// The internal screen of the gameboy is 256*256
const SCREEN_HEIGHT: usize = 256;
const SCREEN_WIDTH: usize = 256;

// But only 166*144 pixels are getting to the LCD screen
const LCD_HEIGHT: usize = 166;
const LCD_WIDTH: usize = 144;

const TILE_MAP_SIZE: usize = 0x400;
const TILE_DATA_SIZE: usize = 0x1000;

// The LCD screen, as part of the GameBoy API.  Holds the logical screen of four
// shades, and all the video-related registers.
pub struct LCD {
  pub pixels: [Shade; SCREEN_HEIGHT * SCREEN_WIDTH],

  control: Control,

  // Status register (STAT)
  // ly_coincidence_interrupt: bool,
  // mode2_oam_interrupt: bool,
  // mode1_vblank_interrupt: bool,
  // mode0_hblank_interrupt: bool,
  // coincidence_flag: bool,
  // mode_flag: Mode,

  // Position and scrolling
  pub scroll_y: u8,
  pub scroll_x: u8,
  y_coordinate: u8,
  // y_compare: u8,
  // window_y: u8,
  // window_x: u8,

  // Palettes
  bg_palette: Palette,
  // object_palette_0: [Shade; 3],
  // object_palette_1: [Shade; 3],

  // Video memory
  pub tile_data_1: [u8; TILE_DATA_SIZE],
  tile_data_2: [u8; TILE_DATA_SIZE],
  pub bg_map_1: [u8; TILE_MAP_SIZE],
  bg_map_2: [u8; TILE_MAP_SIZE],

}

// Control register (LCDC)

#[derive(Debug)]
struct Control {
  lcd_enable: bool,
  window_tile_map: bool,
  window_enable: bool,
  bg_window_tile_data: bool,
  bg_tile_map: bool,
  sprite_size: bool,
  sprite_enable: bool,
  bg_enable: bool,
}

impl Control {
  fn new() -> Self {
    Control::from(0)
  }
}

impl From<u8> for Control {
  fn from(w: u8) -> Self {
    Control {
      lcd_enable:          (w & 0x80) > 0,
      window_tile_map:     (w & 0x40) > 0,
      window_enable:       (w & 0x20) > 0,
      bg_window_tile_data: (w & 0x10) > 0,
      bg_tile_map:         (w & 0x08) > 0,
      sprite_size:         (w & 0x04) > 0,
      sprite_enable:       (w & 0x02) > 0,
      bg_enable:           (w & 0x01) > 0,
    }
  }
}

impl<'a> From<&'a Control> for u8 {
  fn from(c: &Control) -> u8 {
    (if c.lcd_enable          { 1 } else { 0 }) << 7 |
    (if c.window_tile_map     { 1 } else { 0 }) << 6 |
    (if c.window_enable       { 1 } else { 0 }) << 5 |
    (if c.bg_window_tile_data { 1 } else { 0 }) << 4 |
    (if c.bg_tile_map         { 1 } else { 0 }) << 3 |
    (if c.sprite_size         { 1 } else { 0 }) << 2 |
    (if c.sprite_enable       { 1 } else { 0 }) << 1 |
    (if c.bg_enable           { 1 } else { 0 })
  }
}

// Four shades of gray ought to be enough for anyone
#[derive(Copy, Clone, Debug)]
pub enum Shade {
  White,
  LightGray,
  DarkGray,
  Black,
}

// FIXME: Surely there's a way to avoid spelling that out
impl From<u8> for Shade {
  fn from(w: u8) -> Self {
    match w {
      0 => Shade::White,
      1 => Shade::LightGray,
      2 => Shade::DarkGray,
      3 => Shade::Black,
      _ => unreachable!(),
    }
  }
}

impl From<Shade> for u8 {
  fn from(s: Shade) -> u8 {
    match s {
      Shade::White     => 0,
      Shade::LightGray => 1,
      Shade::DarkGray  => 2,
      Shade::Black     => 3,
    }
  }
}

impl Shade {
  fn as_u8(self) -> u8 {
    self.into()
  }

  pub fn as_intensity(self) -> u8 {
    match self {
      Shade::White     => 255,
      Shade::LightGray => 170,
      Shade::DarkGray  => 85,
      Shade::Black     => 0,
    }
  }
}

#[derive(Copy, Clone, Debug)]
struct Palette { data: [Shade; 4] }

impl Palette {
  fn new() -> Self {
    Palette {
      data: [
        Shade::White,
        Shade::LightGray,
        Shade::DarkGray,
        Shade::Black
      ]
    }
  }

  fn shade(&self, c: ColorNumber) -> Shade {
    match c {
      ColorNumber::C0 => self.data[0],
      ColorNumber::C1 => self.data[1],
      ColorNumber::C2 => self.data[2],
      ColorNumber::C3 => self.data[3],
    }
  }
}

impl From<u8> for Palette {
  fn from(w: u8) -> Self {
    Palette {
      data: [
        ((w >> 0) & 0b11).into(),
        ((w >> 2) & 0b11).into(),
        ((w >> 4) & 0b11).into(),
        ((w >> 6) & 0b11).into(),
      ]
    }
  }
}

impl From<Palette> for u8 {
  fn from(p: Palette) -> u8 {
    // Have to use .as_u8 because I can't figure how to typecheck .into
    p.data[3].as_u8()  << 6
      | p.data[2].as_u8() << 4
      | p.data[1].as_u8() << 2
      | p.data[0].as_u8() << 0
  }
}

#[derive(Copy, Clone)]
enum Mode {
  HBlank,
  VBlank,
  Reading_OAM,
  Reading_OAM_and_VRAM,
}

impl LCD {
  pub fn new() -> Self {
    LCD {
      pixels: [Shade::White; SCREEN_HEIGHT * SCREEN_WIDTH],

      control: Control::new(),

      scroll_y: 0,
      scroll_x: 0,
      y_coordinate: 0,

      bg_palette: Palette::new(),

      tile_data_1: [0; 0x1000],
      tile_data_2: [0; 0x1000],
      bg_map_1: [0; 0x400],
      bg_map_2: [0; 0x400],
    }
  }

  pub fn read(&self, addr: u16) -> u8 {
    match addr {
      0xFF40 => (&self.control).into(),
      0xFF42 => self.scroll_x,
      0xFF43 => self.scroll_y,
      0xFF44 => self.y_coordinate,
      0xFF47 => self.bg_palette.into(),
      _ => unreachable!(),
    }
  }

  pub fn write(&mut self, addr: u16, w: u8) {
    match addr {
      0xFF40 => self.control = w.into(),
      0xFF42 => self.scroll_x = w,
      0xFF43 => self.scroll_y = w,
      0xFF44 => self.y_coordinate = w,
      0xFF47 => self.bg_palette = w.into(),
      _ => unreachable!(),
    }

    match addr {
      0xFF40 => {
        println!("Wrote {:x} to LCDC", w);
        println!("{:?}", self.control);
      }
      _ => ()
    };
  }

  // FIXME: Hmm, can't actually use these functions since they lead to borrowing
  // errors, even though putting the same code inline works -_-
  // fn get_bg_tile_map(&self) -> &[u8; TILE_MAP_SIZE] {
  //   match self.control.bg_tile_map {
  //     false => &self.bg_map_1,
  //     true => &self.bg_map_2,
  //   }
  // }

  // fn get_tile_data(&self) -> &[u8; TILE_DATA_SIZE] {
  //   match self.control.bg_window_tile_data {
  //     false => &self.tile_data_1,
  //     true => &self.tile_data_2,
  //   }
  // }

  pub fn clear_background(&mut self) {
    for i in 0..self.pixels.len() {
      self.pixels[i] = Shade::White;
    }
  }

  // Draw the 32*32 background pixels onto the internal screen
  pub fn draw_background(&mut self, tile_map: &[u8], tile_data: &[u8]) {
    // Fetch drawing data from memory
    // let tile_map = match self.control.bg_tile_map {
    //     false => &self.bg_map_1,
    //     true => &self.bg_map_2,
    // };

    // let tile_data = match self.control.bg_window_tile_data {
    //   false => &self.tile_data_1,
    //   true => &self.tile_data_2,
    // };

    // Draw line by line
    for y in 0..SCREEN_HEIGHT {
      for x in 0..SCREEN_WIDTH {
        // x and y are pixel coordinates, but we need to know which tile to draw
        // there

        // Since tiles are 8*8 pixels, ty and tx are the coordinates of the tile
        // on a 32*32 screen
        let ty = y / 8;
        let tx = x / 8;

        // ti is the index of the tile to draw
        let ti = tile_map[ty * 32 + tx] as usize;

        // and t is the tile (16 bytes) to draw from
        let t = &tile_data[ti*16..ti*16+16];

        // Which pixel to take from t?  Tile data is arranged as 16 bytes, 2
        // bytes per line.

        // tl is the line number to take from the tile (0-7)
        let tl = y % 8;

        // and l has the two bytes we need
        let l = &t[tl * 2..tl * 2 + 2];

        // Now we must extract the color number from the bytes

        // b is the bit to take from the line
        let b = 7 - (x % 8);

        let low = (l[1] >> b) & 0x1;
        let high = (l[0] >> b) & 0x1;

        // And we have a 2bit color number
        let c = ColorNumber::from(high << 1 | low);

        // From there we need a shade
        let s = self.bg_palette.shade(c);

       // To put the pixel on the screen, we need the final coordinates after
       // adding scroll and wrapping
        let px = (x as u8).wrapping_sub(self.scroll_x) as usize;
        let py = (y as u8).wrapping_sub(self.scroll_y) as usize;
        self.pixels[py * SCREEN_WIDTH + px] = s;
      }
    }
  }

  pub fn tiles(&self, pattern_table: &[u8]) -> Vec<Tile> {
    pattern_table.chunks(16).map(|tile_data| {
      self.tile_from(tile_data)
    }).collect()
  }

  fn tile_from(&self, data: &[u8]) -> Tile {
    // Combine each line (2 bytes) of each tile as follows:
    //
    // 01011011  <- low byte
    // 01000111  <- high byte
    // --------
    // 02011233  <- vec of u2 (into vec of ColorNumber)
    Tile {
      data: data.chunks(2).flat_map(|line| {
        let mut low = line[0];
        let mut high = line[1];
        let mut res = Vec::new();
        for _ in 0..8 {
          res.push(ColorNumber::from(((high & 1) << 1) + (low & 1)));
          low >>= 1;
          high >>= 1;
        }
        res.reverse();
        res
      })
      // Now we need to map the color codes (0,1,2,3) to an actual shade.
      // That's the palette's job
        .map(|c| self.bg_palette.shade(c))
      // And now we map shades to actual pixel intensity in an u8
        .map(|s| s.as_intensity())
      // and return
        .collect()
    }
  }

  pub fn draw_tiles(&self, tiles: &[Tile], screen: &mut Screen) {
    screen.draw(|pixels| {
      // Now we need to put them at their proper place into the screen.  Each tile
      // has 8*8 pixels, and that tile should be displayed on 8 different lines,
      // rather than all the pixels on the same line as the way it was mapped in
      // memory.

      // TODO: there should be a way to use an iterator to look into the tiles
      // data in the correct order, so you could just loop over it and pump it
      // into the pixels array in order.  Basically, moving the following
      // counting logic to the iterator construction.

      // Coordinates for the pixels in the screen
      let mut sx = 0;
      let mut sy = 0;
      for t in tiles {
        // Coordinates for pixels in the tile data
        for ty in 0..8 {
          for tx in 0..8 {
            pixels[sy * 256 + sx] = t.data[ty * 8 + tx];
            sx += 1;
          }
          sy += 1;
          sx -= 8;
        }

        // Another tile, reset sx/sy
        sy -= 8;
        sx += 8;

        // But if we are on the edge of the screen already, then advance to the
        // next line
        if sx > 255 {
          sx = 0;
          sy += 8;
        }
      }
    })
  }
}


// Valid color number used by tiles
#[derive(Copy, Clone)]
enum ColorNumber {
  C0, C1, C2, C3
}

impl From<u8> for ColorNumber {
  fn from(n: u8) -> ColorNumber {
    match n {
      0 => ColorNumber::C0,
      1 => ColorNumber::C1,
      2 => ColorNumber::C2,
      3 => ColorNumber::C3,
      _ => unreachable!(),
    }
  }
}

// A tile is 8 lines of 8 color numbers.
pub struct Tile {
  pub data: Vec<u8>,
}




// The background is a 256*256 pixels surface for drawing 32*32 tiles from the
// tile map.  It can be scrolled, and is wrapped in both axes.
struct Background {
  pixels: [u8; 256 * 256],
  scroll_x: u8,
  scroll_y: u8,
}

impl Background {

  pub fn get_pixel(&self, x: u8, y: u8) -> u8 {
    // x = x.wrapping_add(self.scroll_x);
    // y = y.wrapping_add(self.scroll_y);

    // Find the sprite
    x
  }
}
