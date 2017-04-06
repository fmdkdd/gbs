use gb::apu::flag::Flag;

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
  enabled: Flag,
  dac_enabled: Flag,

  // Frequency
  period: u16,
  frequency: u16,

  // Length
  length_enabled: Flag,
  length_counter: u16,

  // Volume (no envelope)
  volume: Volume,

  // Samples
  samples: [u8; 16],
  sample_nibble: usize,
  sample_buffer: u8,
}

impl Wave {
  pub fn new() -> Self {
    Wave {
      enabled: Flag::Off,
      dac_enabled: Flag::Off,
      period: 0,
      frequency: 0,
      length_enabled: Flag::Off,
      length_counter: 0,
      volume: Volume::Zero,
      samples: [0; 16],
      sample_nibble: 0,
      sample_buffer: 0,
    }
  }

  pub fn is_enabled(&self) -> bool {
    bool::from(self.enabled)
  }

  pub fn is_dac_enabled(&self) -> bool {
    bool::from(self.dac_enabled)
  }

  pub fn read(&self, reg: Register) -> u8 {
    use self::Register::*;

    match reg {
      NR30 => (self.dac_enabled as u8) << 7,
      NR31 => 0, // write-only
      NR32 => (self.volume as u8) << 5,
      NR33 => 0, // write-only
      NR34 => ((self.length_enabled as u8) << 6),
    }
  }

  pub fn write(&mut self, reg: Register, w: u8) {
    use self::Register::*;

    match reg {
      NR30 => {
        self.dac_enabled = Flag::from((w >> 7) > 0);
        // Any time the DAC is off, the channel is disabled
        if !self.is_dac_enabled() {
          self.enabled = Flag::Off;
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
        self.length_enabled = Flag::from((w & 0x40) > 0);

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
    self.enabled = Flag::On;

    if self.length_counter == 0 {
      self.length_counter = 256;
    }

    self.period = (2048 - self.frequency) * 2;
    self.sample_nibble = 0;
    // sample_buffer is not refilled on trigger
  }

  pub fn clock_length(&mut self) {
    if bool::from(self.length_enabled) && self.length_counter > 0 {
      self.length_counter -= 1;
      if self.length_counter == 0 {
        self.enabled = Flag::Off;
      }
    }
  }

  pub fn clock_frequency(&mut self) {
    if self.period > 0 {
      self.period -= 1;
    } else {
      self.period = (2048 - self.frequency) * 2;

      self.sample_nibble = (self.sample_nibble + 1) % 32;
      self.sample_buffer = self.get_current_sample();
    }
  }

  fn get_current_sample(&self) -> u8 {
    let s = self.samples[self.sample_nibble / 2];

    // Samples are 4bit, so get the right nibble
    if self.sample_nibble % 2 == 0 {
      s >> 4
    } else {
      s & 0x0F
    }
  }

  // Return a value in [0,15]
  fn waveform_output(&self) -> u8 {
    self.sample_buffer
  }

  // Return a value in [0,15]
  fn volume_output(&self) -> u8 {
    if self.is_enabled() {
      // Shift by volume code
      self.waveform_output() >> (self.volume as u8)
    } else {
      0
    }
  }

  // Return a value in [-1.0,+1.0]
  pub fn dac_output(&self) -> f32 {
    if self.is_dac_enabled() {
      let s = self.volume_output() as f32;
      s / 7.5 - 1.0
    } else {
      0.0
    }
  }
}
