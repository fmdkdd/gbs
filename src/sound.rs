// Favor double precision over speed.  But can be changed here.
type F = f64;
use std::f64;
const PI: F = f64::consts::PI;

// 44.1KHz ought to be enough for everyone
pub const SAMPLE_RATE: F = 44100.0;

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

enum Sweep {
  DECREASE, INCREASE
}

enum ShiftRegisterWidth {
  W_15BITS, W_7BITS
}

trait Signal {
  fn get_sample(&mut self, time: F) -> F;
}

pub struct Square1 {
  wave: SquareWave,
  length: Length,
  volume: Volume,
}

impl Square1 {
  pub fn new() -> Self {
    Square1 {
      wave: SquareWave::new(),
      length: Length { length: 0.0 },
      volume: Volume::new(),
    }
  }

  // Frequency, in Hz
  pub fn set_frequency(&mut self, freq: F) {
    self.wave.set_freq(freq);
  }

  // Length in seconds
  pub fn set_length(&mut self, length: F) {
    self.length.length = length;
  }

  pub fn set_pulse_width(&mut self, pulse_width: F) {
    self.wave.pulse_width= pulse_width;
  }

  // Volume in [0,1]
  // Period > 0
  pub fn set_volume_init(&mut self, volume: F, period: F) {
    self.volume.init_volume = (volume * 15.0) as u8;
    self.volume.period = period;
    self.volume.init();
    self.volume.reload();
  }

  // Length in seconds
  pub fn get_samples_for(&mut self, length: F) -> Vec<F> {
    // How many samples do we need?
    let len = length * SAMPLE_RATE;

    (0..(len as u32))
      .map(|s| self.get_sample((s as F) / SAMPLE_RATE))
      .collect()
  }
}

impl Signal for Square1 {
  fn get_sample(&mut self, time: F) -> F {
    self.wave.get_sample(time)
      * self.length.get_sample(time)
      * self.volume.get_sample(time)
  }
}

struct SquareWave {
  freq: F,
  pulse_width: F,
}

impl SquareWave {
  fn new() -> Self {
    SquareWave {
      freq: 0.0,
      pulse_width: 0.5,
    }
  }

  fn set_freq(&mut self, freq: F) {
    self.freq = freq;
  }
}

impl Signal for SquareWave {
  fn get_sample(&mut self, time: F) -> F {
    fourier_square(time, self.freq, self.pulse_width)
  }
}

struct Length {
  length: F,
}

impl Signal for Length {
  fn get_sample(&mut self, time: F) -> F {
    if self.length > 0.0 {
      self.length -= 1.0 / SAMPLE_RATE;
      1.0
    } else {
      0.0
    }
  }
}

struct Volume {
  init_volume: u8,
  sweep: Sweep,
  period: F,

  // Internal
  volume: u8,
  counter: F,
}

impl Volume {
  fn new() -> Self {
    Volume {
      init_volume: 15,
      sweep: Sweep::DECREASE,
      period: 1.0,
      volume: 15,
      counter: 0.0,
    }
  }

  fn init(&mut self) {
    self.volume = self.init_volume;
  }

  fn reload(&mut self) {
    self.counter = self.period / 64.0;
  }
}

impl Signal for Volume {
  fn get_sample(&mut self, time: F) -> F {
    if self.counter > 0.0 {
      self.counter -= 1.0 / SAMPLE_RATE;
      if self.counter <= 0.0 {
        match self.sweep {
          Sweep::DECREASE => self.volume -= 1,
          Sweep::INCREASE => self.volume += 1,
        }
        self.volume &= 0xF;
        if self.volume > 0 {
          self.reload()
        }
      }
    }
    (self.volume as F / 15.0)
  }
}

// Sawtooth made from subtracting even harmonics from odd harmonics in the
// Fourier series.
// freq in Hz
// phase_shift in [0,1]
fn fourier_sawtooth(time: F, freq: F, phase_shift: F) -> F {
  // Sum the Fourier series
  (1u32..)
    .map(|h| h as F)
    // Limit to frequencies below Nyquist barrier
    .take_while(|h| (h * freq) < (SAMPLE_RATE / 2.0))
    .map(|h| {
      let s = F::sin(2.0 * PI * h * (freq * time + phase_shift)) / h;
      if (h as u32) % 2 == 0 { -s }
      else { s }
    }).sum()
}

// Square wave from subtracting two sawtooth waves
// freq in Hz
// pulse_width in [0,1]
fn fourier_square(time: F, freq: F, pulse_width: F) -> F {
  let saw1 = fourier_sawtooth(time, freq, 0.0);
  let saw2 = fourier_sawtooth(time, freq, pulse_width);
  (saw2 - saw1) / 2.0
}
