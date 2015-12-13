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
  pub sp: u16,
  pub pc: u16,
  ram: [u8; RAM_LENGTH],

  cycles: u64,
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
      cycles: 0,
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
    let ret = self.read(self.pc);
    self.pc += 1;
    ret
  }

  fn read(&self, addr: u16) -> u8 {
    self.ram[addr as usize]
  }

  fn write(&mut self, addr: u16, x: u8) {
    self.ram[addr as usize] = x;
  }

  pub fn run(&mut self, cycles: u64) {
    macro_rules! to_u16 {
      ($a:expr, $b:expr) => (($a as u16) << 8 | ($b as u16));
    }

    macro_rules! ld_r1_r2 {
      // Order of these is important.  Match on first argument,
      // then discriminate on second.
      ((c), $r2:ident, $cycles:expr) => ({
        let addr = 0xFF00 | (self.c as u16);
        let x = self.$r2;
        self.write(addr, x);
        self.cycles += $cycles;
      });

      ((n n), $r2:ident, $cycles:expr) => ({
        let l = self.read_pc();
        let h = self.read_pc();
        let addr = to_u16!(h, l);
        let x = self.$r2;
        self.write(addr, x);
        self.cycles += $cycles;
      });

      (($rh:ident $rl:ident), n, $cycles:expr) => ({
        let n = self.read_pc();
        let addr = to_u16!(self.$rh, self.$rl);
        self.write(addr, n);
        self.cycles += $cycles;
      });

      (($rh:ident $rl:ident), $r2:ident, $cycles:expr) => ({
        let addr = to_u16!(self.$rh, self.$rl);
        let x = self.$r2;
        self.write(addr, x);
        self.cycles += $cycles;
      });

      ($r1:ident, (c), $cycles:expr) => ({
        let addr = 0xFF00 | (self.c as u16);
        self.$r1 = self.read(addr);
        self.cycles += $cycles;
      });

      ($r1:ident, (n n), $cycles:expr) => ({
        let l = self.read_pc();
        let h = self.read_pc();
        let addr = to_u16!(h, l);
        self.$r1 = self.read(addr);
        self.cycles += $cycles;
      });

      ($r1:ident, ($rh:ident $rl:ident), $cycles:expr) => ({
        let addr = to_u16!(self.$rh, self.$rl);
        self.$r1 = self.read(addr);
        self.cycles += $cycles;
      });

      ($r1:ident, n, $cycles:expr) => ({
        let n = self.read_pc();
        self.$r1 = n;
        self.cycles += $cycles;
      });

      ($r1:ident, $r2:ident, $cycles:expr) => ({
        self.$r1 = self.$r2;
        self.cycles += $cycles;
      });

    }

    macro_rules! ldd {
      ($r1:ident, ($rh:ident $rl:ident), $cycles:expr) => ({
        let mut addr = to_u16!(self.$rh, self.$rl);
        self.$r1 = self.read(addr);
        addr -= 1;
        let (h, l) = from_u16!(addr);
        self.$rh = h;
        self.$rl = l;
        self.cycles += $cycles;
      });
    }

    macro_rules! from_u16 {
      ($hl:expr) => ( (($hl >> 8) as u8, $hl as u8) )
    }

    self.cycles = 0;

    while self.cycles < cycles {
      let opcode = self.read_pc();
      match opcode {

        // LD nn, n
        0x06 => ld_r1_r2!(b, n, 8),
        0x0E => ld_r1_r2!(c, n, 8),
        0x16 => ld_r1_r2!(d, n, 8),
        0x1E => ld_r1_r2!(e, n, 8),
        0x26 => ld_r1_r2!(h, n, 8),
        0x2E => ld_r1_r2!(l, n, 8),

        // LD r1, r2
        // 0x7F => ld_r1_r2!(a, a, 4), // TODO: implement as NOP
        0x78 => ld_r1_r2!(a, b, 4),
        0x79 => ld_r1_r2!(a, c, 4),
        0x7A => ld_r1_r2!(a, d, 4),
        0x7B => ld_r1_r2!(a, e, 4),
        0x7C => ld_r1_r2!(a, h, 4),
        0x7D => ld_r1_r2!(a, l, 4),
        0x7E => ld_r1_r2!(a, (h l), 8),

        // 0x40 => ld_r1_r2!(b, b, 4),
        0x41 => ld_r1_r2!(b, c, 4),
        0x42 => ld_r1_r2!(b, d, 4),
        0x43 => ld_r1_r2!(b, e, 4),
        0x44 => ld_r1_r2!(b, h, 4),
        0x45 => ld_r1_r2!(b, l, 4),
        0x46 => ld_r1_r2!(b, (h l), 8),

        0x48 => ld_r1_r2!(c, b, 4),
        // 0x49 => ld_r1_r2!(c, c, 4),
        0x4A => ld_r1_r2!(c, d, 4),
        0x4B => ld_r1_r2!(c, e, 4),
        0x4C => ld_r1_r2!(c, h, 4),
        0x4D => ld_r1_r2!(c, l, 4),
        0x4E => ld_r1_r2!(c, (h l), 8),

        0x50 => ld_r1_r2!(d, b, 4),
        0x51 => ld_r1_r2!(d, c, 4),
        // 0x52 => ld_r1_r2!(d, d, 4),
        0x53 => ld_r1_r2!(d, e, 4),
        0x54 => ld_r1_r2!(d, h, 4),
        0x55 => ld_r1_r2!(d, l, 4),
        0x56 => ld_r1_r2!(d, (h l), 8),

        0x58 => ld_r1_r2!(e, b, 4),
        0x59 => ld_r1_r2!(e, c, 4),
        0x5A => ld_r1_r2!(e, d, 4),
        // 0x5B => ld_r1_r2!(e, e, 4),
        0x5C => ld_r1_r2!(e, h, 4),
        0x5D => ld_r1_r2!(e, l, 4),
        0x5E => ld_r1_r2!(e, (h l), 8),

        0x60 => ld_r1_r2!(h, b, 4),
        0x61 => ld_r1_r2!(h, c, 4),
        0x62 => ld_r1_r2!(h, d, 4),
        0x63 => ld_r1_r2!(h, e, 4),
        // 0x64 => ld_r1_r2!(h, h, 4),
        0x65 => ld_r1_r2!(h, l, 4),
        0x66 => ld_r1_r2!(h, (h l), 8),

        0x68 => ld_r1_r2!(l, b, 4),
        0x69 => ld_r1_r2!(l, c, 4),
        0x6A => ld_r1_r2!(l, d, 4),
        0x6B => ld_r1_r2!(l, e, 4),
        0x6C => ld_r1_r2!(l, h, 4),
        // 0x6D => ld_r1_r2!(l, l, 4),
        0x6E => ld_r1_r2!(l, (h l), 8),

        0x70 => ld_r1_r2!((h l), b, 8),
        0x71 => ld_r1_r2!((h l), c, 8),
        0x72 => ld_r1_r2!((h l), d, 8),
        0x73 => ld_r1_r2!((h l), e, 8),
        0x74 => ld_r1_r2!((h l), h, 8),
        0x75 => ld_r1_r2!((h l), l, 8),
        0x36 => ld_r1_r2!((h l), n, 12),

        // LD A,n
        0x0A => ld_r1_r2!(a, (b c), 8),
        0x1A => ld_r1_r2!(a, (d e), 8),
        0xFA => ld_r1_r2!(a, (n n), 16),
        0x3E => ld_r1_r2!(a, n, 8),

        // LD n,A
        0x47 => ld_r1_r2!(b, a, 4),
        0x4F => ld_r1_r2!(c, a, 4),
        0x57 => ld_r1_r2!(d, a, 4),
        0x5F => ld_r1_r2!(e, a, 4),
        0x67 => ld_r1_r2!(h, a, 4),
        0x6F => ld_r1_r2!(l, a, 4),
        0x02 => ld_r1_r2!((b c), a, 8),
        0x12 => ld_r1_r2!((d e), a, 8),
        0x77 => ld_r1_r2!((h l), a, 8),
        0xEA => ld_r1_r2!((n n), a, 16),

        // LD A,(C)
        0xF2 => ld_r1_r2!(a, (c), 8),

        // LD (C),A
        0xE2 => ld_r1_r2!((c), a, 8),

        // LDD A,(HL)
        0x3A => ldd!(a, (h l), 8),

        _ => panic!(format!("Unknown opcode {:x}", opcode))
      }
    }
  }
}
