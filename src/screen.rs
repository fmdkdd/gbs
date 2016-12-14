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

// An array of pixels to draw upon, backed by a Glium display
pub struct Screen {
  width: u32,
  height: u32,
  pixels: Vec<u8>,
  program: Program,
  vertex_buffer: VertexBuffer<Vertex>,
  index_buffer: IndexBuffer<u16>,
  pixel_buffer: PixelBuffer<u8>,
  texture: Texture2d,
}

impl Screen {

  // Initialize the stuff we need to redraw on the screen.  We just use a single
  // texture drawn on a rectangle that fits the window.
  pub fn new<F: Facade>(display: &F, width: u32, height: u32) -> Screen {
    let program = Program::from_source(
      display,
      include_str!("shader/vertex.glsl"),
      include_str!("shader/fragment.glsl"),
      None).unwrap();

    // One nice rectangle to draw the texture upon.
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

    // The buffer holds the pixel values of the texture that will be sent to the
    // GPU
    let pixel_buffer = PixelBuffer::new_empty(display, (width * height) as usize);
    let texture = Texture2d::empty_with_format(display,
                                               UncompressedFloatFormat::U8,
                                               MipmapsOption::NoMipmap,
                                               width,
                                               height).unwrap();

    // Init the texture with a zeroed pixel buffer
    pixel_buffer.write(&vec![0u8; pixel_buffer.get_size()]);
    texture.main_level().raw_upload_from_pixel_buffer(
      pixel_buffer.as_slice(),
      0..width,
      0..height,
      0..1);

    Screen {
      width: width,
      height: height,
      pixels: vec![0u8; (width * height) as usize],
      program: program,
      vertex_buffer: vertex_buffer,
      index_buffer: index_buffer,
      pixel_buffer: pixel_buffer,
      texture: texture,
    }
  }

  // Update the virtual pixel screen with a closure
  pub fn draw<F>(&mut self, draw_fn: F) where F: Fn(&mut [u8]) {
    draw_fn(&mut self.pixels);
  }

  // Update the pixel buffer from the virtual pixel screen, upload the pixel
  // buffer to the texture, and execute a draw call on the supplied frame.
  pub fn repaint<S: Surface>(&mut self, frame: &mut S) {
    self.pixel_buffer.write(&self.pixels);

    // TODO: Maybe create new textures?
    // Should test with full speed to see if it impacts the frame time.
    self.texture.main_level().raw_upload_from_pixel_buffer(
      self.pixel_buffer.as_slice(),
      0..self.width,
      0..self.height,
      0..1);

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
}
