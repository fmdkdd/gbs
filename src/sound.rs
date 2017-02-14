use time::Duration;

struct Counter {
  enabled: bool,
  counter: u8,
}

impl Counter {
  pub fn is_enabled(&self) -> bool {
    enabled
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
  W_15BITS, 7BITS
}

// TODO: most of these are not 8 bit numbers
struct Square1 {
  // Sweep register
  // sweep_period: u8,
  // sweep_direction: SweepDirection,
  // sweep_shift: u8,

  // wave_pattern_duty: u8,
  // length_load: u8,

  // initial_volume: u8,
  // volume_sweep_direction: SweepDirection,
  // sweep_length: u8,

  frequency_period: u16,
  // shift_clock_freq: u8,
  // shift_register_width: ShiftRegisterWidth,
  // freq_dividing_ratio: u8,
}

impl Square1 {


  pub fn set_frequency(&mut self, freq: u16) {
    self.frequency_period = (2048 - freq) * 4;
  }

  pub fn set_length_enable(&mut self ) {
  }

  pub fn clock(&mut self) -> Vec<u8> {

  }

  pub fn run(&mut self, dt: Duration) {

  }
}
