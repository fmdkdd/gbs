use std::f32;

struct Counter {
  enabled: bool,
  counter: u8,
}

impl Counter {
  pub fn is_enabled(&self) -> bool {
    self.enabled
  }

  pub fn reload(&mut self, value: u8) {
    self.counter = value;
  }

  pub fn trigger(&mut self) {
    self.enabled = true;
    if self.counter == 0 {
      self.counter = 64;
    }
  }

  pub fn clock(&mut self) {
    if self.enabled && self.counter > 0 {
      self.counter -= 1;
      if self.counter == 0 {
        self.enabled = false;
      }
    }
  }
}

enum SweepDirection {
  DECREASE, INCREASE
}

enum ShiftRegisterWidth {
  W_15BITS, W_7BITS
}

// TODO: most of these are not 8 bit numbers
pub struct Square1 {
  // Sweep register
  // sweep_period: u8,
  // sweep_direction: SweepDirection,
  // sweep_shift: u8,

  // wave_pattern_duty: u8,
  // length_load: u8,

  // initial_volume: u8,
  // volume_sweep_direction: SweepDirection,
  // sweep_length: u8,

  freq: u16,
  length: u16,

  // shift_clock_freq: u8,
  // shift_register_width: ShiftRegisterWidth,
  // freq_dividing_ratio: u8,
}

impl Square1 {
  pub fn new() -> Self {
    Square1 {
      freq: 0,
      length: 0,
    }
  }

  pub fn set_frequency(&mut self, freq: u16) {
    // self.freq = (2048 - freq) * 4;
    self.freq = freq;
  }

  pub fn set_length(&mut self, length: u16) {
    self.length = length;
  }

  pub fn clock(&mut self) -> Vec<f32> {
    fourier_square(440.0, 0.020, 44100.0, 0.5)
  }

  pub fn run(&mut self, ms: u32) -> Vec<f32> {
    let mut elapsed = 0;
    let mut samples = Vec::new();

    while elapsed < ms {
      samples.append(&mut self.clock());
      elapsed += 20; // MAGIC
    }

    samples
  }
}


// Sawtooth made from subtracting even harmonics from odd harmonics in the
// Fourier series
fn fourier_sawtooth(rate: f32, length: f32, sample_rate: f32, phase_shift: f32) -> Vec<f32> {
  // How many samples do we need?
  let samples = length * sample_rate;

  // How many periods per sample?
  let ratio = rate / sample_rate;

  (0..(samples as u32)).map(|s| {
    // Sum the Fourier series until the Nyquist barrier
    (1u32..)
      .map(|h| h as f32)
    // Limit to frequencies below Nyquist
      .take_while(|h| (h * rate) < (sample_rate / 2.0))
      .map(|h| {
        // t in [0,1] is a time index into a wave period
        let t = ratio * (s as f32);
        let s = f32::sin(2.0 * f32::consts::PI * h * (t + phase_shift)) / h;
        if (h as u32) % 2 == 0 { -s }
        else { s }
      }).sum()
  }).collect()
}

// Square wave from subtracting two sawtooth waves
fn fourier_square(rate: f32, length: f32, sample_rate: f32, pulse_width: f32) -> Vec<f32> {
  let saw = fourier_sawtooth(rate, length, sample_rate, 0.0);

  // How many samples to offset?  For a 50/50 square, we need to offset by half
  // a period.
  let offset = (sample_rate / rate * pulse_width) as usize;

  // Another way to do that is to create two sawtooth waves and use the
  // phase_shift parameter

  saw.iter().zip(saw.iter().skip(offset)).map(|(a,b)| a - b).collect()
}
