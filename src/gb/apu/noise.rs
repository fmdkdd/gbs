use gb::apu::flag::Flag;
use gb::apu::pulse::Sweep;

#[derive(Debug)]
pub enum Register {
  NR41,
  NR42,
  NR43,
  NR44,
}

pub struct Noise {
  enabled: Flag,
  dac_enabled: Flag,

  // Frequency
  period: u32,
  clock_shift: u8,
  width_mode: u8,
  divisor_code: u8,

  // Random bits
  lfsr: u16,

  // Length
  length_enabled: Flag,
  length_counter: u8,

  // Envelope
  volume: u8,
  volume_init: u8,
  volume_counter: u8,
  volume_period: u8,
  volume_sweep: Sweep,
}

impl Noise {
  pub fn new() -> Self {
    Noise {
      enabled: Flag::Off,
      dac_enabled: Flag::Off,
      period: 0,
      clock_shift: 0,
      width_mode: 0,
      divisor_code: 0,
      lfsr: 0,
      length_enabled: Flag::Off,
      length_counter: 0,
      volume: 0,
      volume_init: 0,
      volume_counter: 0,
      volume_period: 0,
      volume_sweep: Sweep::Decrease,
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
      NR41 => 0, // write-only

      NR42 => self.volume_init << 4
        | (self.volume_sweep as u8) << 3
        | self.volume_period,

      NR43 => self.clock_shift << 4
        | self.width_mode << 3
        | self.divisor_code,

      NR44 => (self.length_enabled as u8) << 6,
    }
  }

  pub fn write(&mut self, reg: Register, w: u8) {
    use self::Register::*;

    match reg {
      NR41 => {
        self.length_counter = 64 - w;
      },

      NR42 => {
        self.volume_init = w >> 4;
        self.volume_sweep = Sweep::from_u8((w >> 3) & 0x1).unwrap();
        self.volume_period = w & 0x7;

        // The upper 5 bits of NR_2 are zero control the DAC
        self.dac_enabled = Flag::from((w >> 3) > 0);

        // Any time the DAC is off, the channel is disabled
        if !self.is_dac_enabled() {
          self.enabled = Flag::Off;
        }
      }

      NR43 => {
        self.clock_shift = w >> 4;
        self.width_mode = (w >> 3) & 0x1;
        self.divisor_code = w & 0x7;
      },

      NR44 => {
        self.length_enabled = Flag::from((w & 0x40) > 0);

        if w & 0x80 > 0 {
          self.trigger();
        }
      }
    }
  }

  fn get_period(&self) -> u32 {
    let divisor = match self.divisor_code {
      0 => 8,
      1 => 16,
      2 => 32,
      3 => 48,
      4 => 64,
      5 => 80,
      6 => 96,
      7 => 112,
      _ => unreachable!(),
    };
    divisor << self.clock_shift
  }

  pub fn trigger(&mut self) {
    self.enabled = Flag::On;

    if self.length_counter == 0 {
      self.length_counter = 64;
    }

    self.period = self.get_period();
    // LFSR is 15bit max
    self.lfsr = 0x7FFF;

    self.volume_counter = self.volume_period;
    self.volume = self.volume_init;
  }

  pub fn clock_length(&mut self) {
    if bool::from(self.length_enabled) && self.length_counter > 0 {
      self.length_counter -= 1;
      if self.length_counter == 0 {
        self.enabled = Flag::Off;
      }
    }
  }

  pub fn clock_envelope(&mut self) {
    if self.volume_period > 0 {
      if self.volume_counter > 0 {
        self.volume_counter -= 1;
      } else {
        let new_volume = match self.volume_sweep {
          Sweep::Decrease => self.volume.wrapping_sub(1),
          Sweep::Increase => self.volume + 1,
        };

        if new_volume <= 15 {
          self.volume = new_volume;
          self.volume_counter = self.volume_period;
        }
      }
    }
  }

  pub fn clock_frequency(&mut self) {
    if self.period > 0 {
      self.period -= 1;
    } else {
      self.period = self.get_period();

      let bit = (self.lfsr ^ (self.lfsr >> 1)) & 1;
      self.lfsr >>= 1;
      // Put result of XOR into the now-empty high bit (bit 14)
      self.lfsr |= bit << 14;
      // If width mode is 1, the XOR result is ALSO put into bit 6
      if self.width_mode == 1 {
        self.lfsr = (bit << 6) | (self.lfsr & (!0x40));
      }
    }
  }

  // Return 0 or 1
  fn waveform_output(&self) -> u8 {
    ((!self.lfsr) & 1) as u8
  }

  // Return a value in [0,15]
  fn volume_output(&self) -> u8 {
    if self.is_enabled() {
      self.waveform_output() * self.volume
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
