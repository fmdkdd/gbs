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

pub const SAMPLE_RATE: f32 = 44100.0;

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
    self.length = ((length as f32) / 1000.0 * SAMPLE_RATE) as u16;
  }

  pub fn get_sample(&mut self) -> f32 {
    let sample;

    if self.length > 0 {
      sample = fourier_square(self.length as f32, self.freq as f32, 0.5);
      self.length -= 1;
    } else {
      sample = 0.0;
    }
    sample
  }

  pub fn get_samples_for(&mut self, ms: u32) -> Vec<f32> {
    let mut samples = Vec::new();

    // How many samples do we need?
    let len = ((ms as f32) / 1000.0 * SAMPLE_RATE) as u32;

    for _ in 0..len {
      samples.push(self.get_sample());
    }

    samples
  }
}

// Sawtooth made from subtracting even harmonics from odd harmonics in the
// Fourier series
fn fourier_sawtooth(time: f32, rate: f32, phase_shift: f32) -> f32 {
  // How many periods per sample?
  let ratio = rate / SAMPLE_RATE;

  // Sum the Fourier series until the Nyquist barrier
  (1u32..)
    .map(|h| h as f32)
  // Limit to frequencies below Nyquist
    .take_while(|h| (h * rate) < (SAMPLE_RATE / 2.0))
    .map(|h| {
      let t = ratio * time + phase_shift;
      let s = f32::sin(2.0 * f32::consts::PI * h * t) / h;
      if (h as u32) % 2 == 0 { -s }
      else { s }
    }).sum()
}

// Square wave from subtracting two sawtooth waves
fn fourier_square(time: f32, rate: f32, pulse_width: f32) -> f32 {
  let saw1 = fourier_sawtooth(time, rate, 0.0);
  let saw2 = fourier_sawtooth(time, rate, pulse_width);
  (saw2 - saw1) / 2.0
}
