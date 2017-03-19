use gb::apu::Register;

const DUTY_WAVEFORMS : [[u8; 8]; 4] = [
  [0,0,0,0,0,0,0,1],
  [1,0,0,0,0,0,0,1],
  [1,0,0,0,0,1,1,1],
  [0,1,1,1,1,1,1,0],
];

#[derive(Copy,Clone)]
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

enum Sweep {
  Decrease,
  Increase,
}

impl Sweep {
  fn from_u8(v: u8) -> Option<Self> {
    match v {
      0 => Some(Sweep::Decrease),
      1 => Some(Sweep::Increase),
      _ => None,
    }
  }
}

pub struct Pulse {
  enabled: bool,

  period: u16,
  frequency: u16,
  duty: Duty,
  duty_idx: u8,

  length_counter: u8,

  volume: u8,
  volume_init: u8,
  volume_counter: u8,
  volume_period: u8,
  volume_sweep: Sweep,
}

impl Pulse {
  pub fn new() -> Self {
    Pulse {
      enabled: false,
      period: 0,
      frequency: 0,
      duty: Duty::Half,
      duty_idx: 0,
      length_counter: 0,
      volume: 0,
      volume_init: 0,
      volume_counter: 0,
      volume_period: 0,
      volume_sweep: Sweep::Decrease,
    }
  }

  pub fn read(&self, reg: Register) -> u8 {
    use self::Register::*;

    match reg {
      NR14 | NR24 => (if self.enabled { 1 } else { 0 } << 6) | 0xBF,

      _ => 0xFF,
    }
  }

  pub fn write(&mut self, reg: Register, w: u8) {
    use self::Register::*;

    match reg {
      NR10 => {
      },

      NR11 | NR21 => {
        self.duty = Duty::from_u8(w >> 6).unwrap();
        self.length_counter = 64 - (w & 0x3F);
      },

      NR12 | NR22 => {
        self.volume_init = w >> 4;
        self.volume_sweep = Sweep::from_u8((w >> 3) & 0x1).unwrap();
        self.volume_period = w & 0x7;
      }

      NR13 | NR23 => {
        self.frequency = (self.frequency & 0x0700)
          | w as u16;
      },

      NR14 | NR24 => {
        self.frequency = (self.frequency & 0xFF)
          | ((w & 0x7) as u16) << 8;
        self.enabled = (w & 0x40) > 0;

        if w & 0x80 > 0 {
          self.trigger();
        }
      }
    }
  }

  pub fn trigger(&mut self) {
    self.enabled = true;
    if self.length_counter == 0 {
      self.length_counter = 64;
    }
    self.period = (2048 - self.frequency) * 4;
    self.volume_counter = self.volume_period;
    self.volume = self.volume_init;
  }

  pub fn clock_length(&mut self) {
    if self.length_counter > 0 {
      self.length_counter -= 1;
    } else {
      self.enabled = false;
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

  }

  pub fn clock_frequency(&mut self) {
    if self.period > 0 {
      self.period -= 1;
    } else {
      self.period = (2048 - self.frequency) * 4;
      self.duty_idx = (self.duty_idx + 1) % 8;
    }
  }

  pub fn output(&self) -> u8 {
    if self.enabled {
      DUTY_WAVEFORMS[self.duty as usize][self.duty_idx as usize]
        * self.volume
    } else {
      0
    }
  }
}
