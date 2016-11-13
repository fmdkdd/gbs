use glium::backend::Facade;
use glium::index::PrimitiveType;
use glium::{Surface, VertexBuffer, IndexBuffer, Program};
use glium::texture::{UncompressedFloatFormat, MipmapsOption};
use glium::texture::texture2d::Texture2d;
use glium::texture::pixel_buffer::PixelBuffer;
use glium::uniforms::{MagnifySamplerFilter, MinifySamplerFilter};

#[derive(Copy, Clone)]
struct Vertex {
  position: [f32; 2],
  tex_coords: [f32; 2],
}
implement_vertex!(Vertex, position, tex_coords);

pub const SCREEN_HEIGHT: usize = 256;
pub const SCREEN_WIDTH: usize = 256;

pub struct Screen {
  pixels: [u8; SCREEN_HEIGHT * SCREEN_WIDTH],
  program: Program,
  vertex_buffer: VertexBuffer<Vertex>,
  index_buffer: IndexBuffer<u16>,
  pixel_buffer: PixelBuffer<u8>,
  texture: Texture2d,
}

impl Screen {
  pub fn new<F: Facade>(display: &F) -> Screen {
    let program = Program::from_source(
      display,
      include_str!("shader/vertex.glsl"),
      include_str!("shader/fragment.glsl"),
      None).unwrap();

    // One nice rectangle to hold the texture
    // Texture coordinates are upside-down.
    let vertices = [
      Vertex { position: [-1.0, -1.0], tex_coords: [0.0, 1.0] },
      Vertex { position: [-1.0,  1.0], tex_coords: [0.0, 0.0] },
      Vertex { position: [ 1.0,  1.0], tex_coords: [1.0, 0.0] },
      Vertex { position: [ 1.0, -1.0], tex_coords: [1.0, 1.0] }
    ];

    let vertex_buffer = VertexBuffer::immutable(display, &vertices).unwrap();
    let index_buffer = IndexBuffer::immutable(
      display, PrimitiveType::TriangleStrip, &[1u16, 2, 0, 3]).unwrap();

    // The buffer to hold the pixel values
    let pixel_buffer = PixelBuffer::new_empty(
      display, SCREEN_WIDTH * SCREEN_HEIGHT);
    pixel_buffer.write(&vec![0u8; pixel_buffer.get_size()]);

    let texture = Texture2d::empty_with_format(display,
                                               UncompressedFloatFormat::U8,
                                               MipmapsOption::NoMipmap,
                                               SCREEN_WIDTH as u32,
                                               SCREEN_HEIGHT as u32).unwrap();

    texture.main_level().raw_upload_from_pixel_buffer(
      pixel_buffer.as_slice(),
      0..SCREEN_WIDTH as u32,
      0..SCREEN_HEIGHT as u32, 0..1);

    Screen {
      pixels: [0; SCREEN_HEIGHT * SCREEN_WIDTH],
      program: program,
      vertex_buffer: vertex_buffer,
      index_buffer: index_buffer,
      pixel_buffer: pixel_buffer,
      texture: texture,
    }
  }

  pub fn clear(&mut self) {
    for p in self.pixels.iter_mut() {
      *p = 0;
    }
  }

  pub fn repaint<S: Surface>(&mut self, frame: &mut S) {
    self.pixel_buffer.write(&self.pixels);

    // TODO: Maybe create new textures?
    // Should test with full speed to see if it impacts the frame time.
    self.texture.main_level().raw_upload_from_pixel_buffer(
      self.pixel_buffer.as_slice(),
      0..SCREEN_WIDTH as u32,
      0..SCREEN_HEIGHT as u32, 0..1);

    let uniforms = uniform! {
      tex: self.texture.sampled()
        .minify_filter(MinifySamplerFilter::Nearest)
        .magnify_filter(MagnifySamplerFilter::Nearest),
    };

    frame.draw(&self.vertex_buffer,
               &self.index_buffer,
               &self.program,
               &uniforms, &Default::default()).unwrap();
  }

  pub fn draw_pixel(&mut self, p: u8, x: usize, y: usize) {
    let x = x % SCREEN_WIDTH;
    let y = y % SCREEN_HEIGHT;

    let pos = y * SCREEN_WIDTH + x;
    self.pixels[pos] = p *  64;
  }

  pub fn draw_sprite(&mut self, x: usize, y: usize, sprite: &[u8]) {
    let width = 8;
    let height = sprite.len() / 8;

    for yy in 0..height {
      for xx in 0..width {
        self.draw_pixel(sprite[yy * width + xx], x + xx, y + yy);
      }
    }
  }

  pub fn draw_line(&mut self, x: u8, y: u8, line: Vec<u8>) {
    let mut xx = x;
    for p in line {
      self.draw_pixel(p, xx as usize, y as usize);
      xx = xx.saturating_add(1);
    }
  }

  pub fn combine(up: u8, low: u8) -> Vec<u8> {
    let mut line = Vec::new();
    let mut upp = up;
    let mut loww = low;
    for _ in 0..8 {
      line.push((upp & 1) * 2 + (loww & 1));
      upp >>= 1;
      loww >>= 1;
    }

    line.reverse();
    line
  }

  pub fn draw_tile(&mut self, x: u8, y: u8, tile: &[u8]) {
    let mut i = 0;
    let mut yy = y;
    while i < tile.len() - 1 {
      self.draw_line(x, yy, Self::combine(tile[i], tile[i + 1]));
      i += 2;
      yy = yy.saturating_add(1);
    }
  }

  pub fn draw_tile_table(&mut self, table: &[u8]) {
    let mut i = 0;
    let mut x = 0;
    let mut y = 0;
    while i < table.len() - 16 {
      self.draw_tile(x, y, &table[i..i+16]);
      i += 16;
      x = x.saturating_add(8);
      if x == 255 {
        x = 0;
        y = y.saturating_add(8);
      }
    }
  }
}
