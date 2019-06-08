use gb::apu::flag::Flag;

#[derive(Debug)]
pub enum Register {
  NR10,
  NR11,
  NR12,
  NR13,
  NR14,
  NR21,
  NR22,
  NR23,
  NR24,
}

const DUTY_WAVEFORMS : [[u8; 8]; 4] = [
  [0,0,0,0,0,0,0,1],
  [1,0,0,0,0,0,0,1],
  [1,0,0,0,0,1,1,1],
  [0,1,1,1,1,1,1,0],
];

#[derive(Debug,Copy,Clone)]
enum Duty {
  HalfQuarter,
  Quarter,
  Half,
  ThreeQuarters,
}

impl Duty {
  // Can't use From trait as this conversion can fail
  fn from_u8(v: u8) -> Option<Self> {
    match v {
      0 => Some(Duty::HalfQuarter),
      1 => Some(Duty::Quarter),
      2 => Some(Duty::Half),
      3 => Some(Duty::ThreeQuarters),
      _ => None,
    }
  }
}

#[derive(Debug,Copy,Clone)]
pub enum Sweep {
  Decrease,
  Increase,
}

impl Sweep {
  pub fn from_u8(v: u8) -> Option<Self> {
    match v {
      0 => Some(Sweep::Decrease),
      1 => Some(Sweep::Increase),
      _ => None,
    }
  }
}

pub struct Pulse {
  enabled: Flag,
  dac_enabled: Flag,

  // Frequency + duty
  period: u16,
  frequency: u16,
  duty: Duty,
  duty_idx: u8,

  // Length
  length_enabled: Flag,
  length_counter: u8,

  // Envelope
  volume: u8,
  volume_init: u8,
  volume_counter: u8,
  volume_period: u8,
  volume_sweep: Sweep,

  // Sweep
  sweep_shifts: u8,
  sweep_direction: Sweep,
  sweep_time: u8,
  sweep_counter: u8,
}

impl Pulse {
  pub fn new() -> Self {
    Pulse {
      enabled: Flag::Off,
      dac_enabled: Flag::Off,
      period: 0,
      frequency: 0,
      duty: Duty::Half,
      duty_idx: 0,
      length_enabled: Flag::Off,
      length_counter: 0,
      volume: 0,
      volume_init: 0,
      volume_counter: 0,
      volume_period: 0,
      volume_sweep: Sweep::Decrease,
      sweep_shifts: 0,
      sweep_direction: Sweep::Increase,
      sweep_time: 0,
      sweep_counter: 0,
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
      NR10 => 0,

      NR11 | NR21 => (self.duty as u8) << 6,
      // Length is write-only

      NR12 | NR22 => self.volume_init << 4
        | (self.volume_sweep as u8) << 3
        | self.volume_period,

      // Frequency is write-only
      NR13 | NR23 => 0,

      NR14 | NR24 => ((self.length_enabled as u8) << 6),
    }
  }

  pub fn write(&mut self, reg: Register, w: u8) {
    use self::Register::*;

    match reg {
      NR10 => {
        println!("sweep {}", w);
        self.sweep_shifts = w & 0x3;
        self.sweep_direction = match (w >> 3) & 0x1 {
          0 => Sweep::Increase,
          1 => Sweep::Decrease,
          _ => unreachable!(),
        };
        self.sweep_time = (w >> 6) & 0x3;
      },

      NR11 | NR21 => {
        self.duty = Duty::from_u8(w >> 6).unwrap();
        self.length_counter = 64 - (w & 0x3F);
      },

      NR12 | NR22 => {
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

      NR13 | NR23 => {
        self.frequency = (self.frequency & 0x0700) | (w as u16);
      },

      NR14 | NR24 => {
        self.frequency = (self.frequency & 0xFF) | (((w & 0x7) as u16) << 8);
        self.length_enabled = Flag::from((w & 0x40) > 0);

        if w & 0x80 > 0 {
          self.trigger();
        }
      }
    }
  }

  pub fn trigger(&mut self) {
    self.enabled = Flag::On;

    if self.length_counter == 0 {
      self.length_counter = 64;
    }

    self.period = (2048 - self.frequency) * 4;

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

  pub fn clock_sweep(&mut self) {
    if self.sweep_time == 0 {
      return
    }

    if self.sweep_counter > 0 {
      self.sweep_counter -= 1;
    } else {
      self.sweep_counter = self.sweep_time;
      let f = self.frequency;
      let d = f / (1 << self.sweep_shifts);
      self.frequency = match self.sweep_direction {
        Sweep::Increase => f + d,
        Sweep::Decrease => f - d,
      };
    }
  }

  pub fn clock_frequency(&mut self) {
    if self.period > 0 {
      self.period -= 1;
    } else {
      self.period = (2048 - self.frequency) * 4;
      self.duty_idx = (self.duty_idx + 1) % 8;
    }
  }

  // Return 0 or 1
  fn waveform_output(&self) -> u8 {
    DUTY_WAVEFORMS[self.duty as usize][self.duty_idx as usize]
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
