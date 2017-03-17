const DUTY_WAVEFORMS : [[u8; 8]; 4] = [
  [0,0,0,0,0,0,0,1],
  [1,0,0,0,0,0,0,1],
  [1,0,0,0,0,1,1,1],
  [0,1,1,1,1,1,1,0],
];

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
  duty: Duty,
  length_load: u8,
}

impl Pulse {
  fn new() -> Self {
    Pulse {
      duty: Duty::Half,
      length_load: 0,
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
    match addr {
      0xFF11 => {
        self.pulse1.duty = Duty::from_u8(w >> 6).unwrap();
        self.pulse1.length_load = 64 - (w & 0x3F);
      },

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
}
