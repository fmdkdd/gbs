use std::io::Write;

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

struct Pulse {
  enabled: bool,
  period: u16,
  frequency: u16,
  duty: Duty,
  duty_idx: u8,
  length_counter: u8,

  timer_256: u16,
}

impl Pulse {
  fn new() -> Self {
    Pulse {
      enabled: false,
      period: 0,
      frequency: 440,
      duty: Duty::Half,
      duty_idx: 0,
      length_counter: 0,

      timer_256: 0,
    }
  }

  fn trigger(&mut self) {
    self.enabled = true;
    if self.length_counter == 0 {
      self.length_counter = 64;
    }
    self.period = 2048 - self.frequency;
    writeln!(&mut ::std::io::stderr(), "TRIGGER").unwrap();
  }

  fn clock(&mut self) {
    // 256Hz = 16384 cycles
    if self.timer_256 > 0 {
      self.timer_256 -= 1;
    } else {
      self.timer_256 = 16384;
      // Clock length counter
      if self.length_counter > 0 {
        self.length_counter -= 1;
      } else {
        self.enabled = false;
      }
    }

    if self.period > 0 {
      self.period -= 1;
    } else {
      self.period = 2048 - self.frequency;
      self.duty_idx = (self.duty_idx + 1) % 8;
    }
  }

  fn output(&self) -> u8 {
    if self.enabled {
      DUTY_WAVEFORMS[self.duty as usize][self.duty_idx as usize]
    } else {
      0
    }
  }
}

pub struct APU {
  pulse1: Pulse,

  right_enable_ch1: bool,
  right_enable_ch2: bool,
  right_enable_ch3: bool,
  right_enable_ch4: bool,
  left_enable_ch1: bool,
  left_enable_ch2: bool,
  left_enable_ch3: bool,
  left_enable_ch4: bool,
}

impl APU {
  pub fn new() -> Self {
    APU {
      pulse1: Pulse::new(),

      right_enable_ch1: true,
      right_enable_ch2: true,
      right_enable_ch3: true,
      right_enable_ch4: true,
      left_enable_ch1: true,
      left_enable_ch2: true,
      left_enable_ch3: true,
      left_enable_ch4: true,
    }
  }

  pub fn read(&self, addr: u16) -> u8 {
    match addr {
      0xFF14 => (if self.pulse1.enabled { 1 } else { 0 } << 6)
        | 0xBF,

      0xFF25 => (if self.right_enable_ch1 { 1 } else { 0 })
        | (if self.right_enable_ch2 { 1 } else { 0 } << 1)
        | (if self.right_enable_ch3 { 1 } else { 0 } << 2)
        | (if self.right_enable_ch4 { 1 } else { 0 } << 3)
        | (if self.left_enable_ch1 { 1 } else { 0 } << 4)
        | (if self.left_enable_ch2 { 1 } else { 0 } << 5)
        | (if self.left_enable_ch3 { 1 } else { 0 } << 6)
        | (if self.left_enable_ch4 { 1 } else { 0 } << 7),

        _ => 0xFF,
    }
  }

  pub fn write(&mut self, addr: u16, w: u8) {
    writeln!(&mut ::std::io::stderr(), "poke {:x} {:x}", addr, w).unwrap();

    match addr {
      0xFF11 => {
        self.pulse1.duty = Duty::from_u8(w >> 6).unwrap();
        self.pulse1.length_counter = 64 - (w & 0x3F);
      },

      0xFF13 => {
        self.pulse1.frequency |= w as u16;
      },

      0xFF14 => {
        self.pulse1.frequency |= ((w & 0x7) as u16) << 8;
        self.pulse1.enabled = (w & 0x40) > 0;

        if w & 0x80 > 0 {
          self.pulse1.trigger();
        }
      }

      0xFF25 => {
        self.right_enable_ch1 = (w & 0x01) > 0;
        self.right_enable_ch2 = (w & 0x02) > 0;
        self.right_enable_ch3 = (w & 0x04) > 0;
        self.right_enable_ch4 = (w & 0x08) > 0;
        self.left_enable_ch1 = (w & 0x10) > 0;
        self.left_enable_ch2 = (w & 0x20) > 0;
        self.left_enable_ch3 = (w & 0x40) > 0;
        self.left_enable_ch4 = (w & 0x80) > 0;
      }

      _ => {}
    }
  }

  pub fn step(&mut self) {
    self.pulse1.clock();
  }

  // Return a sample in [-1.0,1.0]
  pub fn output(&self) -> f32 {
    let ch1 = (((self.pulse1.output() * 15) as f32) / 7.5) - 1.0;
    ch1
  }
}
