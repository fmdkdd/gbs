mod pulse;

use gb::apu::pulse::Pulse;

use std::io::Write;

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

pub struct APU {
  pulse1: Pulse,

  // TODO: read/write and clock pulse2
  pulse2: Pulse,

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
      pulse2: Pulse::new(),

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
    use self::Register::*;

    match addr {
      0xFF14 => self.pulse1.read(NR14),

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
    use self::Register::*;

    writeln!(&mut ::std::io::stderr(), "poke {:x} {:x}", addr, w).unwrap();

    match addr {
      0xFF10 => self.pulse1.write(NR10, w),
      0xFF11 => self.pulse1.write(NR11, w),
      0xFF12 => self.pulse1.write(NR12, w),
      0xFF13 => self.pulse1.write(NR13, w),
      0xFF14 => self.pulse1.write(NR14, w),

      0xFF16 => self.pulse2.write(NR21, w),
      0xFF17 => self.pulse2.write(NR22, w),
      0xFF18 => self.pulse2.write(NR23, w),
      0xFF19 => self.pulse2.write(NR24, w),

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
    self.pulse1.clock_sweep();
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
