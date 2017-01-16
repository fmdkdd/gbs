// Test the audio queuing interface of SDL2 with the rust bindings

extern crate sdl2;

use sdl2::audio::{AudioSpecDesired};
use std::time::Duration;

enum SquareState {
  UP, DOWN
}

// A squarewave, that's my use case
struct Square {
  period: u16,
  state: SquareState,
  counter: u16,
}

impl Square {
  // There might be a smarter way to do that than looping through each sample
  fn step(&mut self, samples: u16) -> Vec<i16> {
    let mut result = Vec::new();

    for _ in 0..samples {
      match self.state {
        SquareState::UP => result.push(1000),
        SquareState::DOWN => result.push(-1000),
      }

      self.counter += 1;

      if self.counter >= self.period / 2 {
        match self.state {
          SquareState::UP => self.state = SquareState::DOWN,
          SquareState::DOWN => self.state = SquareState::UP,
        }
        self.counter -= self.period / 2;
      }
    }

    result
  }
}

fn main() {
  let sdl_context = sdl2::init().unwrap();
  let audio_subsystem = sdl_context.audio().unwrap();

  // We don't specify the audio format here, because it's auto-resolved using
  // the type annotation to open_queue
  let desired_spec = AudioSpecDesired {
    freq: Some(48000),
    channels: Some(1),
    // buffer size in number of samples (must be a power of two)
    // note: low values tend to cackle
    samples: Some(1024),  // 1024 / 48000 = 21.33ms buffer size
    // From testing, it appear to be twice size of the minimal amount of samples
    // sent to the audio device between frames.
  };

  // <i16> requests an audio format of S16 (native byte order, according to the source)
  let device = audio_subsystem.open_queue::<i16>(None, &desired_spec).unwrap();

  // Can check the format
  println!("{:?}", device.spec());
  // Well, actually it reports 512 samples when I request 1024, am not sure it
  // is correct with the doc, but at least it's coherent with what I get in C.

  let mut sq = Square { period: 227,
                        state: SquareState::DOWN,
                        counter: 0 };

  // Preload the device with enough samples to start
  device.queue(&sq.step(4096));
  device.resume();

  let mut last_queued_samples = (device.size() as i64) / 2;
  const TARGET_SAMPLES : u16 = 4096;

  // Then add enough samples each frame
  loop {
    // How much samples are left?  device.size() reports the number of *bytes*
    // in the buffer, not samples.
    let queued_samples = (device.size() as i64) / 2;

    // How many samples have we sent to the device since the last frame?
    let sent = last_queued_samples - queued_samples;

    // Push the samples for this frame to maintain a constant buffer size
    let new_samples = (TARGET_SAMPLES as i64) - queued_samples;
    if new_samples > 0 {
      device.queue(&sq.step(new_samples as u16));
    }

    // Is this realistic for an emulator?  I would expect to push as many
    // samples as due to the elapsed time between frames.

    // Report
    println!("samples in queue: {}, sent since last frame: {}",
             queued_samples, sent);
    last_queued_samples = (device.size() as i64) / 2;

    std::thread::sleep(Duration::from_millis(1000 / 100));
  }
}
