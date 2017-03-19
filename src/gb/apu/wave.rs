pub enum Register {
  NR30,
  NR31,
  NR32,
  NR33,
  NR34,
}

#[derive(Copy,Clone)]
pub enum Volume {
  Zero,
  Full,
  Half,
  Quarter,
}

impl Volume {
  // Can't use From trait as this conversion can fail
  fn from_u8(v: u8) -> Option<Self> {
    match v {
      0 => Some(Volume::Zero),
      1 => Some(Volume::Full),
      2 => Some(Volume::Half),
      3 => Some(Volume::Quarter),
      _ => None,
    }
  }
}

pub struct Wave {
  enabled: bool,

  dac_power: bool,

  // Frequency
  period: u16,
  frequency: u16,

  // Length
  length_counter: u16,

  // Volume (no envelope)
  volume: Volume,

  // Samples
  samples: [u8; 16],
  sample_nibble: usize,
}

impl Wave {
  pub fn new() -> Self {
    Wave {
      enabled: false,
      dac_power: false,
      period: 0,
      frequency: 0,
      length_counter: 0,
      volume: Volume::Zero,
      samples: [0; 16],
      sample_nibble: 0,
    }
  }

  pub fn read(&self, reg: Register) -> u8 {
    use self::Register::*;

    match reg {
      NR34 => (if self.enabled { 1 } else { 0 } << 6) | 0xBF,

      _ => 0xFF,
    }
  }

  pub fn write(&mut self, reg: Register, w: u8) {
    use self::Register::*;

    match reg {
      NR30 => {
        self.dac_power = if (w >> 7) > 0 { true } else { false };
        if !self.dac_power {
          self.enabled = false;
        }
      },

      NR31 => {
        self.length_counter = 256 - (w as u16);
      },

      NR32 => {
        self.volume = Volume::from_u8((w >> 5) & 0x3).unwrap();
      }

      NR33 => {
        self.frequency = (self.frequency & 0x0700) | (w as u16);
      },

      NR34 => {
        self.frequency = (self.frequency & 0xFF) | (((w & 0x7) as u16) << 8);
        self.enabled = (w & 0x40) > 0;

        if w & 0x80 > 0 {
          self.trigger();
        }
      }
    }
  }

  pub fn write_sample(&mut self, idx: u16, w: u8) {
    self.samples[idx as usize] = w;
  }

  pub fn trigger(&mut self) {
    self.enabled = true;

    if self.length_counter == 0 {
      self.length_counter = 256;
    }

    self.period = (2048 - self.frequency) * 2;
    self.sample_nibble = 0;
  }

  pub fn clock_length(&mut self) {
    if self.length_counter > 0 {
      self.length_counter -= 1;
    } else {
      self.enabled = false;
    }
  }

  pub fn clock_frequency(&mut self) {
    if self.period > 0 {
      self.period -= 1;
    } else {
      self.period = (2048 - self.frequency) * 2;

      self.sample_nibble = (self.sample_nibble + 1) % 32;
    }
  }

  pub fn output(&self) -> u8 {
    if self.enabled && self.dac_power {
      let mut s = self.samples[self.sample_nibble / 2];

      // Samples are 4bit, so get the right nibble
      if self.sample_nibble % 2 == 0 {
        s >>= 4;
      } else {
        s &= 0x0F;
      }

      // Shift by volume
      s >> (self.volume as u8)
    } else {
      0
    }
  }
}
