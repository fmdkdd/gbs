const RAM_LENGTH : usize = 0x10000;

pub struct Cpu {
  pub a: u8,
  b: u8,
  c: u8,
  d: u8,
  e: u8,
  f: u8,
  h: u8,
  l: u8,
  sp: u16,
  pub pc: u16,
  ram: [u8; RAM_LENGTH],
}

impl Cpu {

  pub fn new() -> Cpu {
    Cpu {
      a: 0,
      b: 0,
      c: 0,
      d: 0,
      e: 0,
      f: 0,
      h: 0,
      l: 0,
      sp: 0,
      pc: 0,
      ram: [0; RAM_LENGTH],
    }
  }

  pub fn reset(&mut self) {
    // TODO
  }

  pub fn load_rom(&mut self, rom: &Vec<u8>, offset: usize) {
    // Copy ROM into RAM banks 0 and 1, stopping at 0x7fff, or when ROM is
    // empty.
    let mut ra = offset;
    let ra_max = 0x8000;
    let mut ro = 0;
    let ro_max = rom.len();
    while ro < ro_max && ra < ra_max {
      self.ram[ra] = rom[ro];
      ra += 1;
      ro += 1;
    }
  }

  fn read_pc(&mut self) -> u8 {
    let ret = self.ram[self.pc as usize];
    self.pc += 1;
    ret
  }

  pub fn run(&mut self, cycles: u64) {
    let cycles_count = 0;

    while cycles_count < cycles {
      let opcode = self.read_pc();
      match opcode {
        // 0x06 => ld(b,n)
        _ => panic!(format!("Unknown opcode {:x}", opcode))
      }
    }
  }
}
