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

struct Pulse {
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
  fn new() -> Self {
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

  fn trigger(&mut self) {
    self.enabled = true;
    if self.length_counter == 0 {
      self.length_counter = 64;
    }
    self.period = (2048 - self.frequency) * 4;
    self.volume_counter = self.volume_period;
    self.volume = self.volume_init;
  }

  fn clock_length(&mut self) {
    if self.length_counter > 0 {
      self.length_counter -= 1;
    } else {
      self.enabled = false;
    }
  }

  fn clock_envelope(&mut self) {
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

  fn clock_frequency(&mut self) {
    if self.period > 0 {
      self.period -= 1;
    } else {
      self.period = (2048 - self.frequency) * 4;
      self.duty_idx = (self.duty_idx + 1) % 8;
    }
  }

  fn output(&self) -> u8 {
    if self.enabled {
      DUTY_WAVEFORMS[self.duty as usize][self.duty_idx as usize]
        * self.volume
    } else {
      0
    }
  }
}

pub struct APU {
  pulse1: Pulse,

  frame_seq: FrameSequencer,

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

      frame_seq: FrameSequencer::new(),

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

      0xFF12 => {
        self.pulse1.volume_init = w >> 4;
        self.pulse1.volume_sweep = Sweep::from_u8((w >> 3) & 0x1).unwrap();
        self.pulse1.volume_period = w & 0x7;
      }

      0xFF13 => {
        self.pulse1.frequency = (self.pulse1.frequency & 0x0700)
          | w as u16;
      },

      0xFF14 => {
        self.pulse1.frequency = (self.pulse1.frequency & 0xFF)
          | ((w & 0x7) as u16) << 8;
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

  // Clock APU.  Should be called at GB_FREQ: 1 CPU cycle = 1 APU cycle.
  pub fn step(&mut self) {
    self.pulse1.clock_frequency();

    // Frame sequencer timing:
    //
    // Step Length Ctr  Vol Env   Sweep
    // ------------------------------------
    // 0    Clock       -         -
    // 1    -           -         -
    // 2    Clock       -         Clock
    // 3    -           -         -
    // 4    Clock       -         -
    // 5    -           -         -
    // 6    Clock       -         Clock
    // 7    -           Clock     -
    // ------------------------------------
    // Rate 256 Hz      64 Hz     128 Hz
    if self.frame_seq.clock() {
      self.clock_512();

      if self.frame_seq.frame % 2 == 0 {
        self.clock_256();
      }

      if self.frame_seq.frame % 4 == 2 {
        self.clock_128();
      }

      if self.frame_seq.frame % 8 == 7 {
        self.clock_64();
      }
    }
  }

  fn clock_512(&mut self) {
  }

  fn clock_256(&mut self) {
    self.pulse1.clock_length();
  }

  fn clock_128(&mut self) {
  }

  fn clock_64(&mut self) {
    self.pulse1.clock_envelope();
  }

  // Return a sample in [-1.0,1.0]
  pub fn output(&self) -> f32 {
    let ch1 = ((self.pulse1.output() as f32) / 7.5) - 1.0;
    ch1
  }
}

// 512Hz timer controlling low-frequency modulation units in the APU
struct FrameSequencer {
  frame: u32,
  period: u16,
}

impl FrameSequencer {
  fn new() -> Self {
    FrameSequencer {
      frame: 0,
      // TODO: should period be initially loaded?
      period: 0,
    }
  }

  fn clock(&mut self) -> bool {
    if self.period > 0 {
      self.period -= 1;
      false
    } else {
      self.period = 8192;
      self.frame = self.frame.wrapping_add(1);
      true
    }
  }
}
