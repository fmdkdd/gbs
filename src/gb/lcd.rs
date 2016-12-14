use super::super::screen::Screen;

const LCD_HEIGHT: usize = 166;
const LCD_WIDTH: usize = 144;

// The LCD screen, as part of the GameBoy API
pub struct LCD {
  // pixels: [u8; LCD_HEIGHT * LCD_WIDTH],
  bgp: [Shade; 4],
}

impl LCD {
  pub fn new() -> Self {
    LCD {
      bgp: [Shade::White,
            Shade::LightGray,
            Shade::DarkGray,
            Shade::Black]
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
        .map(|c| self.palette(c))
      // And now we map shades to actual pixel intensity in an u8
        .map(|s| s.into())
      // and return
        .collect()
    }
  }

  fn palette(&self, color: ColorNumber) -> Shade {
    // TODO: get palette from memory
    self.bgp[color as usize]
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

#[derive(Copy,Clone)]
enum Shade {
  White,
  LightGray,
  DarkGray,
  Black,
}

impl From<Shade> for u8 {
  fn from(s: Shade) -> Self {
    match s {
      Shade::White => 255,
      Shade::LightGray => 170,
      Shade::DarkGray => 85,
      Shade::Black => 0,
    }
  }
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
