const RAM_LENGTH : usize = 0x10000;

const Z_FLAG : u8 = 0x80;
const N_FLAG : u8 = 0x40;
const H_FLAG : u8 = 0x20;
const C_FLAG : u8 = 0x10;

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
  pub ram: [u8; RAM_LENGTH],

  interrupts_enabled: bool,
}

macro_rules! to_u16 {
  ($h:expr, $l:expr) => (($h as u16) << 8 | ($l as u16));
}

macro_rules! from_u16 {
  ($hl:expr) => ( (($hl >> 8) as u8, $hl as u8) )
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
      interrupts_enabled: true,
    }
  }

  pub fn reset(&mut self) {
    // TODO
    self.pc = 0x100;
    self.a = 0x0;
    self.b = 0x0;
    self.c = 0x13;
    self.d = 0x0;
    self.e = 0xD8;
    self.h = 0x01;
    self.l = 0x4D;
    self.f = 0x1;
    self.sp = 0xFFFE;
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

  fn set_z(&mut self) { self.f |= Z_FLAG; }
  fn set_n(&mut self) { self.f |= N_FLAG; }
  fn set_h(&mut self) { self.f |= H_FLAG; }
  fn set_c(&mut self) { self.f |= C_FLAG; }
  fn clear_z(&mut self) { self.f &= !Z_FLAG; }
  fn clear_n(&mut self) { self.f &= !N_FLAG; }
  fn clear_h(&mut self) { self.f &= !H_FLAG; }
  fn clear_c(&mut self) { self.f &= !C_FLAG; }
  fn z(&self) -> bool { self.f & Z_FLAG == Z_FLAG }
  fn n(&self) -> bool { self.f & N_FLAG == N_FLAG }
  fn h(&self) -> bool { self.f & H_FLAG == H_FLAG }
  fn c(&self) -> bool { self.f & C_FLAG == C_FLAG }

  fn read_pc(&mut self) -> u8 {
    let ret = self.read(self.pc);
    self.pc = self.pc.wrapping_add(1);
    ret
  }

  fn read_pc_16le(&mut self) -> u16 {
    let l = self.read_pc();
    let h = self.read_pc();
    to_u16!(h, l)
  }

  fn read(&self, addr: u16) -> u8 {
    self.ram[addr as usize]
  }

  fn read_16le(&self, addr: u16) -> u16 {
    let l = self.ram[addr as usize];
    let h = self.ram[(addr + 1) as usize];
    to_u16!(h, l)
  }

  fn write(&mut self, addr: u16, x: u8) {
    if addr == 0xFF01 {
      println!("{}", x as char);
    }

    self.ram[addr as usize] = x;
  }

  fn write_16le(&mut self, addr: u16, x: u16) {
    let (h, l) = from_u16!(x);
    self.ram[addr as usize] = l;
    self.ram[(addr + 1) as usize] = h;
  }

  // Run the next instruction
  pub fn step(&mut self) -> u8 {
    let mut cycles = 0;

    macro_rules! nop {
      () => ({ cycles += 4; });
    }

    macro_rules! ld {
      // Order of these is important.  Match on first argument,
      // then discriminate on second.

      // LD (FF00+n),A
      ((0xFF00 + n), a) => ({
        let n = self.read_pc();
        let addr = 0xFF00 | (n as u16);
        let v = self.a;
        self.write(addr, v);
        cycles += 12;
      });

      // LD (FF00+C),A
      ((0xFF00 + c), a) => ({
        let addr = 0xFF00 | (self.c as u16);
        let v = self.a;
        self.write(addr, v);
        cycles += 8;
      });

      // LD (nn),A
      ((n n), a) => ({
        let addr = self.read_pc_16le();
        let v = self.a;
        self.write(addr, v);
        cycles += 16;
      });

      // LD (nn),SP
      ((n n), sp) => ({
        let addr = self.read_pc_16le();
        let v = self.sp;
        self.write_16le(addr, v);
        cycles += 16; // XXX: not sure
      });

      // LD (HL),n
      ((h l), n) => ({
        let n = self.read_pc();
        let addr = to_u16!(self.h, self.l);
        self.write(addr, n);
        cycles += 12;
      });

      // LD (HL),r
      // LD (BC),A
      // LD (DE),A
      (($rh:ident $rl:ident), $r2:ident) => ({
        let addr = to_u16!(self.$rh, self.$rl);
        let v = self.$r2;
        self.write(addr, v);
        cycles += 8;
      });

      // LD A,(FF00+n)
      (a, (0xFF00 + n)) => ({
        let n = self.read_pc();
        let addr = 0xFF00 | (n as u16);
        self.a = self.read(addr);
        cycles += 12;
      });

      // LD A,(FF00+C)
      (a, (0xFF00 + c)) => ({
        let addr = 0xFF00 | (self.c as u16);
        self.a = self.read(addr);
        cycles += 8;
      });

      // LD A,(nn)
      (a, (n n)) => ({
        let addr = self.read_pc_16le();
        self.a = self.read(addr);
        cycles += 16;
      });

      // LD r,(HL)
      // LD A,(BC)
      // LD A,(DE)
      ($r1:ident, ($rh:ident $rl:ident)) => ({
        let addr = to_u16!(self.$rh, self.$rl);
        self.$r1 = self.read(addr);
        cycles += 8;
      });

      // LD SP,nn
      (sp, nn) => ({
        self.sp = self.read_pc_16le();
        cycles += 12;
      });

      (hl, sp+dd) => ({
        let dd = self.read_pc() as i8;
        let (h, l) = from_u16!(self.sp.wrapping_add(dd as u16));
        self.l = l;
        self.h = h;
        // TODO: flags
        cycles += 12;
      });

      // LD r,n
      ($r1:ident, n) => ({
        let n = self.read_pc();
        self.$r1 = n;
        cycles += 8;
      });

      // LD r,r
      ($r1:ident, $r2:ident) => ({
        self.$r1 = self.$r2;
        cycles += 4;
      });

      // LD SP,HL
      (sp, h l) => ({
        self.sp = to_u16!(self.h, self.l);
        cycles += 8;
      });

      // LD rr,nn
      ($rh:ident $rl:ident, nn) => ({
        self.$rl = self.read_pc();
        self.$rh = self.read_pc();
        cycles += 12;
      });
    }

    macro_rules! ldd {
      // LDD (HL),A
      ((h l), a) => ({
        let mut addr = to_u16!(self.h, self.l);
        let v = self.a;
        self.write(addr, v);
        addr += addr.wrapping_sub(1);
        let (h, l) = from_u16!(addr);
        self.h = h;
        self.l = l;
        cycles += 8;
      });

      // LDD A,(HL)
      (a, (h l)) => ({
        let mut addr = to_u16!(self.h, self.l);
        self.a = self.read(addr);
        addr = addr.wrapping_sub(1);
        let (h, l) = from_u16!(addr);
        self.h = h;
        self.l = l;
        cycles += 8;
      })
    }

    macro_rules! ldi {
      // LDD (HL),A
      ((h l), a) => ({
        let mut addr = to_u16!(self.h, self.l);
        let v = self.a;
        self.write(addr, v);
        addr = addr.wrapping_add(1);
        let (h, l) = from_u16!(addr);
        self.h = h;
        self.l = l;
        cycles += 8;
      });

      // LDD A,(HL)
      (a, (h l)) => ({
        let mut addr = to_u16!(self.h, self.l);
        self.a = self.read(addr);
        addr = addr.wrapping_add(1);
        let (h, l) = from_u16!(addr);
        self.h = h;
        self.l = l;
        cycles += 8;
      })
    }

    macro_rules! push {
      ($rh:ident $rl:ident) => ({
        self.sp = self.sp.wrapping_sub(1);
        let addr = self.sp;
        let v = self.$rh;
        self.write(addr, v);
        self.sp = self.sp.wrapping_sub(1);
        let addr = self.sp;
        let v = self.$rl;
        self.write(addr, v);
        cycles += 16;
      })
    }

    macro_rules! pop {
      ($rh:ident $rl:ident) => ({
        let l = self.read(self.sp);
        self.sp = self.sp.wrapping_add(1);
        let h = self.read(self.sp);
        self.sp = self.sp.wrapping_add(1);
        self.$rl = l;
        self.$rh = h;
        // TODO: flags?
        cycles += 12;
      })
    }

    macro_rules! inc {
      // INC (HL)
      ((h l)) => ({
        let addr = to_u16!(self.h, self.l);
        let v = self.read(addr);
        let r = v.wrapping_add(1);
        self.write(addr, r);
        flags!(z0h-, v, r);
        cycles += 12;
      });

      // INC sp
      (sp) => ({
        self.sp = self.sp.wrapping_add(1);
        cycles += 8;
      });

      // INC r
      ($r:ident) => ({
        let v = self.$r;
        self.$r = v.wrapping_add(1);
        flags!(z0h-, v, self.$r);
        cycles += 4;
      });

      // INC rr
      ($rh:ident $rl:ident) => ({
        let mut rr = to_u16!(self.$rh, self.$rl);
        rr = rr.wrapping_add(1);
        let (h, l) = from_u16!(rr);
        self.$rh = h;
        self.$rl = l;
        cycles += 8;
      });
    }

    macro_rules! dec {
      // DEC (HL)
      ((h l)) => ({
        let addr = to_u16!(self.h, self.l);
        let v = self.read(addr);
        let r = v.wrapping_sub(1);
        self.write(addr, r);
        flags!(z1h-, v, r);
        cycles += 12;
      });

      // DEC sp
      (sp) => ({
        self.sp = self.sp.wrapping_sub(1);
        cycles += 8;
      });

      // DEC r
      ($r:ident) => ({
        let v = self.$r;
        self.$r = v.wrapping_sub(1);
        flags!(z1h-, v, self.$r);
        cycles += 4;
      });

      // DEC rr
      ($rh:ident $rl:ident) => ({
        let mut rr = to_u16!(self.$rh, self.$rl);
        rr = rr.wrapping_sub(1);
        let (h, l) = from_u16!(rr);
        self.$rh = h;
        self.$rl = l;
        cycles += 8;
      });
    }

    macro_rules! add1 {
      ($n:expr) => ({
        let n = $n;
        let mut r = self.a as u16;
        r += n as u16;

        let mut rh = self.a & 0xF;
        rh += n & 0xF;

        flags!(z0hc, r, rh);

        self.a = r as u8;
      });
    }

    macro_rules! add {
      // ADD A,(HL)
      ((h l)) => ({
        let addr = to_u16!(self.h, self.l);
        add1!(self.read(addr));
        cycles += 8;
      });

      // ADD A,n
      (n) => ({
        add1!(self.read_pc());
        cycles += 8;
      });

      // ADD A,r
      ($r:ident) => ({
        add1!(self.$r);
        cycles += 4;
      });

      // ADD HL,SP
      (hl, sp) => ({
        let mut r = to_u16!(self.h, self.l) as u32;
        r += self.sp as u32;

        let mut rh = self.h & 0xF;
        rh += ((self.sp >> 8) & 0xF) as u8;

        flags!(-0hc, r, rh);

        let (h, l) = from_u16!(r as u16);
        self.h = h;
        self.l = l;
        cycles += 8;
      });

      // ADD SP,dd
      (sp, dd) => ({
        let dd = self.read_pc() as i8;
        self.sp = self.sp.wrapping_add(dd as u16);
        // TODO: flags
        cycles += 16;
      });

      // ADD HL,rr
      (hl, $rh:ident $rl:ident) => ({
        let mut r = to_u16!(self.h, self.l) as u32;
        r += to_u16!(self.$rh, self.$rl) as u32;

        let mut rh = self.h & 0xF;
        rh += self.$rh & 0xF;

        flags!(-0hc, r, rh);

        let (h, l) = from_u16!(r as u16);
        self.h = h;
        self.l = l;
        cycles += 8;
      });

    }

    macro_rules! adc1 {
      ($n:expr) => ({
        let n = $n;
        let mut r = self.a as u16;
        r += if self.c() { 1 } else { 0 };
        r += n as u16;

        let mut rh = self.a & 0xF;
        rh += if self.c() { 1 } else { 0 };
        rh += n & 0xF;

        flags!(z0hc, r, rh);

        self.a = r as u8;
      });
    }

    macro_rules! adc {
      // ADC A,(HL)
      ((h l)) => ({
        let addr = to_u16!(self.h, self.l);
        adc1!(self.read(addr));
        cycles += 8;
      });

      // ADC A,n
      (n) => ({
        adc1!(self.read_pc());
        cycles += 8;
      });

      // ADC A,r
      ($r:ident) => ({
        adc1!(self.$r);
        cycles += 4;
      });
    }

    macro_rules! sub1 {
      ($n:expr) => ({
        let n = $n;
        let mut r = self.a as i16;
        r -= n as i16;

        let mut rh = (self.a & 0xF) as i8;
        rh -= (n & 0xF) as i8;

        flags!(z1hc, r, rh);

        self.a = r as u8;
      });
    }

    macro_rules! sub {
      // SUB A,(HL)
      ((h l)) => ({
        let addr = to_u16!(self.h, self.l);
        sub1!(self.read(addr));
        cycles += 8;
      });

      // SUB A,n
      (n) => ({
        sub1!(self.read_pc());
        cycles += 8;
      });

      // SUB A,r
      ($r:ident) => ({
        sub1!(self.$r);
        cycles += 4;
      });
    }

    macro_rules! sbc1 {
      ($n:expr) => ({
        let n = $n;
        let mut r = self.a as i16;
        r -= if self.c() { 1 } else { 0 };
        r -= n as i16;

        let mut rh = (self.a & 0xF) as i8;
        rh -= if self.c() { 1 } else { 0 };
        rh -= (n & 0xF) as i8;

        flags!(z1hc, r, rh);

        self.a = r as u8;
      });
    }

    macro_rules! sbc {
      // SBC A,(HL)
      ((h l)) => ({
        let addr = to_u16!(self.h, self.l);
        sbc1!(self.read(addr));
        cycles += 8;
      });

      // SBC A,n
      (n) => ({
        sbc1!(self.read_pc());
        cycles += 8;
      });

      // SBC A,r
      ($r:ident) => ({
        sbc1!(self.$r);
        cycles += 4;
      });
    }

    macro_rules! and {
      // AND A,(HL)
      ((h l)) => ({
        let addr = to_u16!(self.h, self.l);
        self.a &= self.read(addr);
        flags!(z010, self.a);
        cycles += 8;
      });

      // AND A,n
      (n) => ({
        self.a &= self.read_pc();
        flags!(z010, self.a);
        cycles += 8;
      });

      // AND A,r
      ($r:ident) => ({
        self.a &= self.$r;
        flags!(z010, self.a);
        cycles += 4;
      });
    }

    macro_rules! xor {
      // XOR A,(HL)
      ((h l)) => ({
        let addr = to_u16!(self.h, self.l);
        self.a ^= self.read(addr);
        flags!(z000, self.a);
        cycles += 8;
      });

      // XOR A,n
      (n) => ({
        self.a ^= self.read_pc();
        flags!(z000, self.a);
        cycles += 8;
      });

      // XOR A,r
      ($r:ident) => ({
        self.a ^= self.$r;
        flags!(z000, self.a);
        cycles += 4;
      });
    }

    macro_rules! or {
      // OR A,(HL)
      ((h l)) => ({
        let addr = to_u16!(self.h, self.l);
        self.a |= self.read(addr);
        flags!(z000, self.a);
        cycles += 8;
      });

      // OR A,n
      (n) => ({
        self.a |= self.read_pc();
        flags!(z000, self.a);
        cycles += 8;
      });

      // OR A,r
      ($r:ident) => ({
        self.a |= self.$r;
        flags!(z000, self.a);
        cycles += 4;
      });
    }

    macro_rules! cp1 {
      ($n:expr) => ({
        let n = $n;
        let mut r = self.a as i16;
        r -= n as i16;

        let mut rh = (self.a & 0xF) as i8;
        rh -= (n & 0xF) as i8;

        flags!(z1hc, r, rh);
      });
    }

    macro_rules! cp {
      // CP A,(HL)
      ((h l)) => ({
        let addr = to_u16!(self.h, self.l);
        cp1!(self.read(addr));
        cycles += 8;
      });

      // CP A,n
      (n) => ({
        cp1!(self.read_pc());
        cycles += 8;
      });

      // CP A,r
      ($r:ident) => ({
        cp1!(self.$r);
        cycles += 4;
      });
    }

    macro_rules! rlca {
      () => ({
        // Bit 7 to carry flag
        self.f = (self.a & 0x80) << 4;
        self.a = self.a.rotate_left(1);
        cycles += 4;
      });
    }

    macro_rules! rlc {
      ((h l)) => ({
        let addr = to_u16!(self.h, self.l);
        let mut v = self.read(addr);
        // Bit 7 to carry flag
        self.f = (v & 0x80) << 4;
        v = v.rotate_left(1);
        self.write(addr, v);
        cycles += 16;
      });

      // RLC r
      ($r:ident) => ({
        // Bit 7 to carry flag
        self.f = (self.$r & 0x80) << 4;
        self.$r = self.$r.rotate_left(1);
        cycles += 8;
      });
    }

    macro_rules! rla {
      () => ({
        // Get carry
        let c = (self.f & C_FLAG) >> 4;
        // Bit 7 to carry flag
        self.f = (self.a & 0x80) << 4;
        self.a = (self.a << 1) | c;
        cycles += 4;
      });
    }

    macro_rules! rl {
      ((h l)) => ({
        // Get carry
        let c = (self.f & C_FLAG) >> 4;
        let addr = to_u16!(self.h, self.l);
        let mut v = self.read(addr);
        // Bit 7 to carry flag
        self.f = (v & 0x80) << 4;
        v = (v << 1) | c;
        self.write(addr, v);
        cycles += 16;
      });

      ($r:ident) => ({
        // Get carry
        let c = (self.f & C_FLAG) >> 4;
        // Bit 7 to carry flag
        self.f = (self.$r & 0x80) << 4;
        self.$r = (self.$r << 1) | c;
        cycles += 8;
      });
    }

    macro_rules! rrca {
      () => ({
        // Bit 0 to carry flag
        self.f = (self.a & 0x1) << 4;
        self.a = self.a.rotate_right(1);
        cycles += 4;
      });
    }

    macro_rules! rrc {
      ((h l)) => ({
        let addr = to_u16!(self.h, self.l);
        let mut v = self.read(addr);
        // Bit 0 to carry flag
        self.f = (v & 0x1) << 4;
        v = v.rotate_right(1);
        self.write(addr, v);
        cycles += 16;
      });

      ($r:ident) => ({
        // Bit 0 to carry flag
        self.f = (self.$r & 0x1) << 4;
        self.$r = self.$r.rotate_right(1);
        cycles += 8;
      });
    }

    macro_rules! rra {
      () => ({
        // Get carry as bit 7
        let c = (self.f & C_FLAG) << 3;
        // Bit 0 to carry flag
        self.f = (self.a & 0x1) << 4;
        self.a = c | (self.a >> 1);
        cycles += 4;
      });
    }

    macro_rules! rr {
      ((h l)) => ({
        // Get carry as bit 7
        let c = (self.f & C_FLAG) << 3;
        let addr = to_u16!(self.h, self.l);
        let mut v = self.read(addr);
        // Bit 0 to carry flag
        self.f = (v & 0x1) << 4;
        v = c | (v >> 1);
        self.write(addr, v);
        cycles += 16;
      });

      ($r:ident) => ({
        // Get carry as bit 7
        let c = (self.f & C_FLAG) << 3;
        // Bit 0 to carry flag
        self.f = (self.$r & 0x1) << 4;
        self.$r = c | (self.$r >> 1);
        cycles += 8;
      });
    }

    macro_rules! sla {
      ((h l)) => ({
        let addr = to_u16!(self.h, self.l);
        let mut v = self.read(addr);
        // Bit 7 to carry flag
        self.f = (v & 0x80) << 4;
        v = v << 1;
        self.write(addr, v);
        cycles += 16;
      });

      ($r:ident) => ({
        // Bit 7 to carry flag
        self.f = (self.$r & 0x80) << 4;
        self.$r = self.$r << 1;
        cycles += 8;
      });
    }

    macro_rules! swap {
      ((h l)) => ({
        let addr = to_u16!(self.h, self.l);
        let mut v = self.read(addr);
        // Bit 7 to carry flag
        self.f = (v & 0x80) << 4;
        v = v.rotate_left(4);
        self.write(addr, v);
        cycles += 16;
      });

      ($r:ident) => ({
        // Bit 7 to carry flag
        self.f = (self.$r & 0x80) << 4;
        self.$r = self.$r.rotate_left(4);
        cycles += 8;
      });
    }

    macro_rules! sra {
      ((h l)) => ({
        let addr = to_u16!(self.h, self.l);
        let mut v = self.read(addr);
        // Keep bit 7
        let b7 = v & 0x80;
        // Bit 0 to carry flag
        self.f = (v & 0x1) << 4;
        v = b7 | (v >> 1);
        self.write(addr, v);
        cycles += 16;
      });

      ($r:ident) => ({
        // Keep bit 7
        let b7 = self.$r & 0x80;
        // Bit 0 to carry flag
        self.f = (self.$r & 0x1) << 4;
        self.$r = b7 | (self.$r >> 1);
        cycles += 8;
      });
    }

    macro_rules! srl {
      ((h l)) => ({
        let addr = to_u16!(self.h, self.l);
        let mut v = self.read(addr);
        // Bit 0 to carry flag
        self.f = (v & 0x1) << 4;
        v = v >> 1;
        self.write(addr, v);
        cycles += 16;
      });

      ($r:ident) => ({
        // Bit 0 to carry flag
        self.f = (self.$r & 0x1) << 4;
        self.$r = self.$r >> 1;
        cycles += 8;
      });
    }

    macro_rules! ccf {
      () => ({
        self.f = self.f ^ C_FLAG;
        cycles += 4;
      })
    }

    macro_rules! scf {
      () => ({
        self.f = self.f & C_FLAG;
        cycles += 4;
      })
    }

    macro_rules! di {
      () => ({
        self.interrupts_enabled = false;
        cycles += 4;
      });
    }

    macro_rules! ei {
      () => ({
        self.interrupts_enabled = true;
        cycles += 4;
      });
    }

    macro_rules! flags {
      // First argument stands for znhc flags.
      //   z: set if $r is 0
      //   n: 0 or 1
      //   h: set if nibble overflow
      //   c: set if operation overflow/underflow
      // $v: value before operation
      // $r: value after operation

      (z0h-, $v:expr, $r:expr) => ({
        self.f =
          ((($r == 0) as u8) << 7)
          | (((($v & 0xF) == 0xF) as u8) << 5)
          | (self.f & C_FLAG);
      });

      (z1h-, $v:expr, $r:expr) => ({
        self.f =
          ((($r == 0) as u8) << 7)
          | ((($v == 0) as u8) << 5)
          | N_FLAG
          | (self.f & C_FLAG);
      });

      // $r: result as u16
      // $rh: result of nibble operation as u8
      (z0hc, $r:expr, $rh:expr) => ({
        if ($r & 0x00FF) == 0 {
          self.set_z();
        }
        else {
          self.clear_z();
        }

        self.clear_n();

        if ($rh & 0xF0) > 0 {
          self.set_h();
        }
        else {
          self.clear_h();
        }

        if ($r & 0xFF00) > 0 {
          self.set_c();
        }
        else  {
          self.clear_c();
        }
      });

      // $r: result as u16
      // $rh: result of nibble operation as u8
      (z1hc, $r:expr, $rh:expr) => ({
        if ($r & 0x00FF) == 0 {
          self.set_z();
        }
        else {
          self.clear_z();
        }

        self.set_n();

        if $rh < 0 {
          self.set_h();
        }
        else {
          self.clear_h();
        }

        if $r < 0 {
          self.set_c();
        }
        else  {
          self.clear_c();
        }
      });

      (z010, $r:expr) => ({
        if $r == 0 {
          self.set_z();
        }
        else {
          self.clear_z();
        }

        self.clear_n();
        self.set_h();
        self.clear_c();
      });

      (z000, $r:expr) => ({
        if $r == 0 {
          self.set_z();
        }
        else {
          self.clear_z();
        }

        self.clear_n();
        self.clear_h();
        self.clear_c();
      });

      // $r: u32
      // $rh: u8
      (-0hc, $r:expr, $rh:expr) => ({
        self.clear_n();

        if ($rh & 0xF0) > 0 {
          self.set_h();
        }
        else {
          self.clear_h();
        }

        if ($r & 0xFFFF0000) > 0 {
          self.set_c();
        }
        else {
          self.clear_c();
        }

      });
    }

    macro_rules! jp {
      (nn) => ({
        let addr = self.read_pc_16le();
        self.pc = addr;
        cycles += 16;
      });

      (hl) => ({
        self.pc = to_u16!(self.h, self.l);
        cycles += 4;
      });

      (nz) => ({
        let addr = self.read_pc_16le();
        if (self.f & Z_FLAG) == 0 {
          self.pc = addr;
          cycles += 4;
        }
        cycles += 12;
      });

      (z) => ({
        let addr = self.read_pc_16le();
        if (self.f & Z_FLAG) > 0 {
          self.pc = addr;
          cycles += 4;
        }
        cycles += 12;
      });

      (nc) => ({
        let addr = self.read_pc_16le();
        if (self.f & C_FLAG) == 0 {
          self.pc = addr;
          cycles += 4;
        }
        cycles += 12;
      });

      (c) => ({
        let addr = self.read_pc_16le();
        if (self.f & C_FLAG) > 0 {
          self.pc = addr;
          cycles += 4;
        }
        cycles += 12;
      });
    }

    macro_rules! jr {
      (dd) => ({
        let dd = self.read_pc() as i8;
        self.pc = self.pc.wrapping_add(dd as u16);
        cycles += 12;
      });

      (nz) => ({
        let dd = self.read_pc() as i8;
        if (self.f & Z_FLAG) == 0 {
          self.pc = self.pc.wrapping_add(dd as u16);
          cycles += 4;
        }
        cycles += 8;
      });

      (z) => ({
        let dd = self.read_pc() as i8;
        if (self.f & Z_FLAG) > 0 {
          self.pc = self.pc.wrapping_add(dd as u16);
          cycles += 4;
        }
        cycles += 8;
      });

      (nc) => ({
        let dd = self.read_pc() as i8;
        if (self.f & C_FLAG) == 0 {
          self.pc = self.pc.wrapping_add(dd as u16);
          cycles += 4;
        }
        cycles += 8;
      });

      (c) => ({
        let dd = self.read_pc() as i8;
        if (self.f & C_FLAG) > 0 {
          self.pc = self.pc.wrapping_add(dd as u16);
          cycles += 4;
        }
        cycles += 8;
      });
    }

    macro_rules! call1 {
      ($nn:expr) => ({
        // Push PC to stack
        self.sp = self.sp.wrapping_sub(2);
        let sp = self.sp;
        let pc = self.pc;
        self.write_16le(sp, pc);
        // Jump to nn
        self.pc = $nn;
      });
    }

    macro_rules! call {
      (nn) => ({
        call1!(self.read_pc_16le());
        cycles += 24;
      });

      (nz) => ({
        let addr = self.read_pc_16le();
        if (self.f & Z_FLAG) == 0 {
          call1!(addr);
          cycles += 12;
        }
        cycles += 12;
      });

      (z) => ({
        let addr = self.read_pc_16le();
        if (self.f & Z_FLAG) > 0 {
          call1!(addr);
          cycles += 12;
        }
        cycles += 12;
      });

      (nc) => ({
        let addr = self.read_pc_16le();
        if (self.f & C_FLAG) == 0 {
          call1!(addr);
          cycles += 12;
        }
        cycles += 12;
      });

      (c) => ({
        let addr = self.read_pc_16le();
        if (self.f & C_FLAG) > 0 {
          call1!(addr);
          cycles += 12;
        }
        cycles += 12;
      });
    }

    macro_rules! ret {
      () => ({
        self.pc = self.read_16le(self.sp);
        self.sp = self.sp.wrapping_add(2);
        cycles += 16;
      });

      (nz) => ({
        if (self.f & Z_FLAG) == 0 {
          self.pc = self.read_16le(self.sp);
          self.sp = self.sp.wrapping_add(2);
          cycles += 12;
        }
        cycles += 8;
      });

      (z) => ({
        if (self.f & Z_FLAG) > 0 {
          self.pc = self.read_16le(self.sp);
          self.sp = self.sp.wrapping_add(2);
          cycles += 12;
        }
        cycles += 8;
      });

      (nc) => ({
        if (self.f & C_FLAG) == 0 {
          self.pc = self.read_16le(self.sp);
          self.sp = self.sp.wrapping_add(2);
          cycles += 12;
        }
        cycles += 8;
      });

      (c) => ({
        if (self.f & C_FLAG) > 0 {
          self.pc = self.read_16le(self.sp);
          self.sp = self.sp.wrapping_add(2);
          cycles += 12;
        }
        cycles += 8;
      });
    }

    macro_rules! reti {
      () => ({
        self.pc = self.read_16le(self.sp);
        self.sp = self.sp.wrapping_add(2);
        self.interrupts_enabled = true;
        cycles += 16;
      });
    }

    macro_rules! rst {
      ($n:expr) => ({
        // Jump to $0000 + n
        call1!($n as u16);
        cycles += 16;
      });
    }

    let opcode = self.read_pc();
    match opcode {
      // Following the order of pandoc, to know what I still have left to
      // implement.

      // GMB 8bit loads

      // LD r,r
      0x41 => ld!(b, c),
      0x42 => ld!(b, d),
      0x43 => ld!(b, e),
      0x44 => ld!(b, h),
      0x45 => ld!(b, l),
      0x47 => ld!(b, a),

      0x48 => ld!(c, b),
      0x4A => ld!(c, d),
      0x4B => ld!(c, e),
      0x4C => ld!(c, h),
      0x4D => ld!(c, l),
      0x4F => ld!(c, a),

      0x50 => ld!(d, b),
      0x51 => ld!(d, c),
      0x53 => ld!(d, e),
      0x54 => ld!(d, h),
      0x55 => ld!(d, l),
      0x57 => ld!(d, a),

      0x58 => ld!(e, b),
      0x59 => ld!(e, c),
      0x5A => ld!(e, d),
      0x5C => ld!(e, h),
      0x5D => ld!(e, l),
      0x5F => ld!(e, a),

      0x60 => ld!(h, b),
      0x61 => ld!(h, c),
      0x62 => ld!(h, d),
      0x63 => ld!(h, e),
      0x65 => ld!(h, l),
      0x67 => ld!(h, a),

      0x68 => ld!(l, b),
      0x69 => ld!(l, c),
      0x6A => ld!(l, d),
      0x6B => ld!(l, e),
      0x6C => ld!(l, h),
      0x6F => ld!(l, a),

      0x78 => ld!(a, b),
      0x79 => ld!(a, c),
      0x7A => ld!(a, d),
      0x7B => ld!(a, e),
      0x7C => ld!(a, h),
      0x7D => ld!(a, l),

      // LD B,B
      // LD C,C
      // LD D,D
      // LD E,E
      // LD H,H
      // LD L,L
      // LD A,A
      0x40 => nop!(),
      0x49 => nop!(),
      0x52 => nop!(),
      0x5B => nop!(),
      0x64 => nop!(),
      0x6D => nop!(),
      0x7F => nop!(),

      // LD r,n
      0x06 => ld!(b, n),
      0x0E => ld!(c, n),
      0x16 => ld!(d, n),
      0x1E => ld!(e, n),
      0x26 => ld!(h, n),
      0x2E => ld!(l, n),
      0x3E => ld!(a, n),

      // LD r,(HL)
      0x46 => ld!(b, (h l)),
      0x4E => ld!(c, (h l)),
      0x56 => ld!(d, (h l)),
      0x5E => ld!(e, (h l)),
      0x66 => ld!(h, (h l)),
      0x6E => ld!(l, (h l)),
      0x7E => ld!(a, (h l)),

      // LD (HL),r
      0x70 => ld!((h l), b),
      0x71 => ld!((h l), c),
      0x72 => ld!((h l), d),
      0x73 => ld!((h l), e),
      0x74 => ld!((h l), h),
      0x75 => ld!((h l), l),
      0x77 => ld!((h l), a),

      0x36 => ld!((h l), n),

      0x0A => ld!(a, (b c)),
      0x1A => ld!(a, (d e)),
      0xFA => ld!(a, (n n)),

      0x02 => ld!((b c), a),
      0x12 => ld!((d e), a),
      0xEA => ld!((n n), a),

      0xF0 => ld!(a, (0xFF00 + n)),
      0xE0 => ld!((0xFF00 + n), a),

      0xF2 => ld!(a, (0xFF00 + c)),
      0xE2 => ld!((0xFF00 + c), a),

      0x22 => ldi!((h l), a),
      0x2A => ldi!(a, (h l)),
      0x32 => ldd!((h l), a),
      0x3A => ldd!(a, (h l)),

      // GMB 16bit loads

      // LD rr,nn
      0x01 => ld!(b c, nn),
      0x11 => ld!(d e, nn),
      0x21 => ld!(h l, nn),
      0x31 => ld!(sp, nn),

      0xF9 => ld!(sp, h l),

      // PUSH rr
      0xC5 => push!(b c),
      0xD5 => push!(d e),
      0xE5 => push!(h l),
      0xF5 => push!(a f),

      // POP rr
      0xC1 => pop!(b c),
      0xD1 => pop!(d e),
      0xE1 => pop!(h l),
      0xF1 => pop!(a f),

      // GMB 8bit arithmetic/logical

      // ADD A,r
      0x80 => add!(b),
      0x81 => add!(c),
      0x82 => add!(d),
      0x83 => add!(e),
      0x84 => add!(h),
      0x85 => add!(l),
      0x87 => add!(a),

      0xC6 => add!(n),

      0x86 => add!((h l)),

      // ADC A,r
      0x88 => adc!(b),
      0x89 => adc!(c),
      0x8A => adc!(d),
      0x8B => adc!(e),
      0x8C => adc!(h),
      0x8D => adc!(l),
      0x8F => adc!(a),

      0xCE => adc!(n),

      0x8E => adc!((h l)),

      // SUB A,r
      0x90 => sub!(b),
      0x91 => sub!(c),
      0x92 => sub!(d),
      0x93 => sub!(e),
      0x94 => sub!(h),
      0x95 => sub!(l),
      0x97 => sub!(a),

      0xD6 => sub!(n),

      0x96 => sub!((h l)),

      0x98 => sbc!(b),
      0x99 => sbc!(c),
      0x9A => sbc!(d),
      0x9B => sbc!(e),
      0x9C => sbc!(h),
      0x9D => sbc!(l),
      0x9F => sbc!(a),

      0xDE => sbc!(n),

      0x9E => sbc!((h l)),

      // AND r
      0xA0 => and!(b),
      0xA1 => and!(c),
      0xA2 => and!(d),
      0xA3 => and!(e),
      0xA4 => and!(h),
      0xA5 => and!(l),
      0xA7 => and!(a),

      0xE6 => and!(n),

      0xA6 => and!((h l)),

      // XOR r
      0xA8 => xor!(b),
      0xA9 => xor!(c),
      0xAA => xor!(d),
      0xAB => xor!(e),
      0xAC => xor!(h),
      0xAD => xor!(l),
      0xAF => xor!(a),

      0xEE => xor!(n),

      0xAE => xor!((h l)),

      // OR r
      0xB0 => or!(b),
      0xB1 => or!(c),
      0xB2 => or!(d),
      0xB3 => or!(e),
      0xB4 => or!(h),
      0xB5 => or!(l),
      0xB7 => or!(a),

      0xF6 => or!(n),

      0xB6 => or!((h l)),

      // CP r
      0xB8 => cp!(b),
      0xB9 => cp!(c),
      0xBA => cp!(d),
      0xBB => cp!(e),
      0xBC => cp!(h),
      0xBD => cp!(l),
      0xBF => cp!(a),

      0xFE => cp!(n),

      0xBE => cp!((h l)),

      // INC r
      0x04 => inc!(b),
      0x0C => inc!(c),
      0x14 => inc!(d),
      0x1C => inc!(e),
      0x24 => inc!(h),
      0x2C => inc!(l),
      0x3C => inc!(a),

      0x34 => inc!((h l)),

      // DEC r
      0x05 => dec!(b),
      0x0D => dec!(c),
      0x15 => dec!(d),
      0x1D => dec!(e),
      0x25 => dec!(h),
      0x2D => dec!(l),
      0x3D => dec!(a),

      0x35 => dec!((h l)),

      // TODO: daa
      // TODO: cpl

      // GMB 16bit arithmetic/logical

      0x09 => add!(hl, b c),
      0x19 => add!(hl, d e),
      0x29 => add!(hl, h l),
      0x39 => add!(hl, sp),

      // INC rr
      0x03 => inc!(b c),
      0x13 => inc!(d e),
      0x23 => inc!(h l),
      0x33 => inc!(sp),

      // DEC rr
      0x0B => dec!(b c),
      0x1B => dec!(d e),
      0x2B => dec!(h l),
      0x3B => dec!(sp),

      0xE8 => add!(sp, dd),

      0xF8 => ld!(hl, sp+dd),

      // GMB rotate and shift

      0x07 => rlc!(a),

      0x17 => rl!(a),

      0x0F => rrc!(a),

      0x1F => rr!(a),

      0xCB => {
        let cb_opcode = self.read_pc();
        match cb_opcode {

          // TODO: zero flags for all

          // RLC r
          0x00 => rlc!(b),
          0x01 => rlc!(c),
          0x02 => rlc!(d),
          0x03 => rlc!(e),
          0x04 => rlc!(h),
          0x05 => rlc!(l),
          0x07 => rlc!(a),

          0x06 => rlc!((h l)),

          // RR r
          0x10 => rl!(b),
          0x11 => rl!(c),
          0x12 => rl!(d),
          0x13 => rl!(e),
          0x14 => rl!(h),
          0x15 => rl!(l),
          0x17 => rl!(a),

          0x16 => rl!((h l)),

          // RRC r
          0x08 => rrc!(b),
          0x09 => rrc!(c),
          0x0A => rrc!(d),
          0x0B => rrc!(e),
          0x0C => rrc!(h),
          0x0D => rrc!(l),
          0x0F => rrc!(a),

          0x0E => rrc!((h l)),

          // RR r
          0x18 => rr!(b),
          0x19 => rr!(c),
          0x1A => rr!(d),
          0x1B => rr!(e),
          0x1C => rr!(h),
          0x1D => rr!(l),
          0x1F => rr!(a),

          0x1E => rr!((h l)),

          // SLA r
          0x20 => sla!(b),
          0x21 => sla!(c),
          0x22 => sla!(d),
          0x23 => sla!(e),
          0x24 => sla!(h),
          0x25 => sla!(l),
          0x27 => sla!(a),

          0x26 => sla!((h l)),

          // SWAP r
          0x30 => swap!(b),
          0x31 => swap!(c),
          0x32 => swap!(d),
          0x33 => swap!(e),
          0x34 => swap!(h),
          0x35 => swap!(l),
          0x37 => swap!(a),

          0x36 => swap!((h l)),

          // SRA r
          0x28 => sra!(b),
          0x29 => sra!(c),
          0x2A => sra!(d),
          0x2B => sra!(e),
          0x2C => sra!(h),
          0x2D => sra!(l),
          0x2F => sra!(a),

          0x2E => sra!((h l)),

          // SRL r
          0x38 => srl!(b),
          0x39 => srl!(c),
          0x3A => srl!(d),
          0x3B => srl!(e),
          0x3C => srl!(h),
          0x3D => srl!(l),
          0x3F => srl!(a),

          0x3E => srl!((h l)),

          _ => panic!(format!("Unknown opcode 0xCB{:x}", cb_opcode))
        }
      },

      // GMB CPU control

      0x3F => ccf!(),
      0x37 => scf!(),
      0x00 => nop!(),
      // TODO: halt
      // TODO: stop
      0xF3 => di!(),
      0xFB => ei!(),

      // GMB jumps

      0xC3 => jp!(nn),

      0xE9 => jp!(hl),

      // JP f,nn
      0xC2 => jp!(nz),
      0xCA => jp!(z),
      0xD2 => jp!(nc),
      0xDA => jp!(c),

      0x18 => jr!(dd),

      // JR f,PC+dd
      0x20 => jr!(nz),
      0x28 => jr!(z),
      0x30 => jr!(nc),
      0x38 => jr!(c),

      0xCD => call!(nn),

      // CALL f,nn
      0xC4 => call!(nz),
      0xCC => call!(z),
      0xD4 => call!(nc),
      0xDC => call!(c),

      // RET f
      0xC9 => ret!(),
      0xC0 => ret!(nz),
      0xC8 => ret!(z),
      0xD0 => ret!(nc),
      0xD8 => ret!(c),

      0xD9 => reti!(),

      // RST n
      0xC7 => rst!(0x00),
      0xCF => rst!(0x08),
      0xD7 => rst!(0x10),
      0xDF => rst!(0x18),
      0xE7 => rst!(0x20),
      0xEF => rst!(0x28),
      0xF7 => rst!(0x30),
      0xFF => rst!(0x38),

      // Extra opcodes

      0x08 => ld!((n n), sp),
      0xE3 => nop!(),

      _ => panic!(format!("Unknown opcode 0x{:x}", opcode))
    }

    cycles
  }

  pub fn run_for(&mut self, cycles: u64) {
    let mut c : u64 = 0;

    while c < cycles {
      c += self.step() as u64;
    }
  }

  pub fn run(&mut self) {
    loop {
      self.step();
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  // Some helpers around `assert` with better debugging messages.
  macro_rules! expect_eq {
    ($l: expr, $r: expr) => ({
      let l = $l;
      let r = $r;
      assert!(r == l, "expected {:x}, got {:x}", r, l);
    });
  }

  macro_rules! expect_cycles {
    ($cycles: expr, $n: expr) => ({
      assert!($cycles == $n, "expected {} cycles, got {}", $n, $cycles);
    });
  }

  macro_rules! expect_flag {
    (z, $f: expr, $e: expr) => ({
      let e = if $e == 1 { true } else { false };
      assert!($f == e, "expected zero flag to be {}",
              if e { "set" } else { "clear" });
    });

    (n, $f: expr, $e: expr) => ({
      let e = if $e == 1 { true } else { false };
      assert!($f == e, "expected add/sub flag to be {}",
              if e { "set" } else { "clear" });
    });

    (h, $f: expr, $e: expr) => ({
      let e = if $e == 1 { true } else { false };
      assert!($f == e, "expected half carry flag to be {}",
              if e { "set" } else { "clear" });
    });

    (c, $f: expr, $e: expr) => ({
      let e = if $e == 1 { true } else { false };
      assert!($f == e, "expected carry flag to be {}",
              if e { "set" } else { "clear" });
    });
  }

  // Use macros to generate the test functions, since this is a lot of repetitive
  // code, and we want to test all the opcodes, and all the registers.
  //
  // Have to pass the name of each generated function, since we cannot generate
  // identifiers using macros (one day, maybe).  Still repetitive, but more
  // palatable.
  macro_rules! test_ld {

    // LD (FF00+n),A
    ($name: ident, $opcode: expr, (0xFF00 + n), a) => {
      #[test]
      fn $name() {
        let mut cpu = Cpu::new();

        cpu.pc = 0;
        cpu.ram[0] = $opcode;
        cpu.ram[1] = 0xAD;
        let cycles = cpu.step();

        expect_cycles!(cycles, 12);
      }
    };

    // LD (FF00+c),A
    ($name: ident, $opcode: expr, (0xFF00 + c), a) => {
      #[test]
      fn $name() {
        let mut cpu = Cpu::new();

        cpu.c = 0xAD;

        cpu.pc = 0;
        cpu.ram[0] = $opcode;
        let cycles = cpu.step();

        expect_cycles!(cycles, 8);
      }
    };

    // LD (hl),n
    ($name: ident, $opcode: expr, (hl), n) => {
      #[test]
      fn $name() {
        let mut cpu = Cpu::new();
        let n = 0xBA;

        cpu.h = 0xDE;
        cpu.l = 0xAD;

        cpu.ram[0xDEAD] = 0;
        cpu.pc = 0;
        cpu.ram[0] = $opcode;
        cpu.ram[1] = n;
        let cycles = cpu.step();

        expect_eq!(cpu.ram[0xDEAD], n);
        expect_cycles!(cycles, 12);
      }
    };

    // LD (nn),A
    ($name:ident, $opcode:expr, (n n), $r:ident) => {
      #[test]
      fn $name() {
        let mut cpu = Cpu::new();

        cpu.$r = 0xBA;
        cpu.ram[0xDEAD] = 0;

        cpu.pc = 0;
        cpu.ram[0] = $opcode;
        cpu.ram[1] = 0xAD;      // Little endian
        cpu.ram[2] = 0xDE;
        let cycles = cpu.step();

        expect_eq!(cpu.ram[0xDEAD], 0xBA);
        expect_cycles!(cycles, 16);
      }
    };

    // LD (hl),r
    // LD (BC),A
    // LD (DE),A
    ($name:ident, $opcode:expr, ($rh:ident $rl:ident), $r:ident) => {
      #[test]
      fn $name() {
        let mut cpu = Cpu::new();

        cpu.$r = 0xBA;
        cpu.$rh = 0xDE;
        cpu.$rl = 0xAD;

        cpu.ram[0xDEAD] = 0;
        cpu.pc = 0;
        cpu.ram[0] = $opcode;
        let cycles = cpu.step();

        expect_eq!(cpu.ram[0xDEAD], cpu.$r);
        expect_cycles!(cycles, 8);
      }
    };

    // LD rr,nn
    ($name: ident, $opcode: expr, [$rh:ident $rl:ident], nn) => {
      #[test]
      fn $name() {
        let mut cpu = Cpu::new();

        cpu.$rh = 0;
        cpu.$rl = 0;

        cpu.pc = 0;
        cpu.ram[0] = $opcode;
        cpu.ram[1] = 0xAD;      // Little endian
        cpu.ram[2] = 0xDE;
        let cycles = cpu.step();

        expect_eq!(cpu.$rh, 0xDE);
        expect_eq!(cpu.$rl, 0xAD);
        expect_cycles!(cycles, 12);
      }
    };

    // LD sp,nn
    ($name: ident, $opcode: expr, sp, nn) => {
      #[test]
      fn $name() {
        let mut cpu = Cpu::new();

        cpu.sp = 0;

        cpu.pc = 0;
        cpu.ram[0] = $opcode;
        cpu.ram[1] = 0xAD;      // Little endian
        cpu.ram[2] = 0xDE;
        let cycles = cpu.step();

        expect_eq!(cpu.sp, 0xDEAD);
        expect_cycles!(cycles, 12);
      }
    };

    // LD sp,hl
    ($name: ident, $opcode: expr, sp, hl) => {
      #[test]
      fn $name() {
        let mut cpu = Cpu::new();

        cpu.sp = 0;
        cpu.h = 0xDE;
        cpu.l = 0xAD;

        cpu.pc = 0;
        cpu.ram[0] = $opcode;
        let cycles = cpu.step();

        expect_eq!(cpu.sp, 0xDEAD);
        expect_cycles!(cycles, 8);
      }
    };

    // LD A,(FF00+n)
    ($name: ident, $opcode: expr, a, (0xFF00 + n)) => {
      #[test]
      fn $name() {
        let mut cpu = Cpu::new();

        cpu.pc = 0;
        cpu.ram[0] = $opcode;
        cpu.ram[1] = 0xAD;
        let cycles = cpu.step();

        expect_cycles!(cycles, 12);
      }
    };

    // LD A,(FF00+c)
    ($name: ident, $opcode: expr, a, (0xFF00 + c)) => {
      #[test]
      fn $name() {
        let mut cpu = Cpu::new();

        cpu.c = 0xAD;

        cpu.pc = 0;
        cpu.ram[0] = $opcode;
        let cycles = cpu.step();

        expect_cycles!(cycles, 8);
      }
    };

    // LD A,(nn)
    ($name: ident, $opcode: expr, $r:ident, (n n)) => {
      #[test]
      fn $name() {
        let mut cpu = Cpu::new();

        cpu.$r = 0;
        cpu.ram[0xDEAD] = 0xBA;

        cpu.pc = 0;
        cpu.ram[0] = $opcode;
        cpu.ram[1] = 0xAD;      // Little endian
        cpu.ram[2] = 0xDE;
        let cycles = cpu.step();

        expect_eq!(cpu.$r, 0xBA);
        expect_cycles!(cycles, 16);
      }
    };

    // LD r,(hl)
    // LD A,(BC)
    // LD A,(DE)
    ($name: ident, $opcode: expr, $r:ident, ($rh:ident $rl:ident)) => {
      #[test]
      fn $name() {
        let mut cpu = Cpu::new();

        cpu.$r = 0;
        cpu.$rh = 0xDE;
        cpu.$rl = 0xAD;

        cpu.ram[0xDEAD] = 0xBA;
        cpu.pc = 0;
        cpu.ram[0] = $opcode;
        let cycles = cpu.step();

        expect_eq!(cpu.$r, 0xBA);
        expect_cycles!(cycles, 8);
      }
    };

    // LD r,n
    ($name: ident, $opcode: expr, $r:ident, n) => {
      #[test]
      fn $name() {
        let mut cpu = Cpu::new();
        let n = 0xBA;

        cpu.$r = 0;

        cpu.pc = 0;
        cpu.ram[0] = $opcode;
        cpu.ram[1] = n;
        let cycles = cpu.step();

        expect_eq!(cpu.$r, n);
        expect_cycles!(cycles, 8);
      }
    };

    // LD r,r
    ($name: ident, $opcode: expr, $r1:ident, $r2:ident) => {
      #[test]
      fn $name() {
        let mut cpu = Cpu::new();

        cpu.$r1 = 0;
        cpu.$r2 = 0xBA;

        cpu.pc = 0;
        cpu.ram[0] = $opcode;
        let cycles = cpu.step();

        expect_eq!(cpu.$r1, cpu.$r2);
        expect_cycles!(cycles, 4);
      }
    };
  }

  macro_rules! test_ldi {

    // LDI (HL),A
    ($name: ident, $opcode: expr, (h l), a) => {
      #[test]
      fn $name() {
        let mut cpu = Cpu::new();

        cpu.a = 0xBA;
        cpu.ram[0xDEAD] = 0;
        cpu.h = 0xDE;
        cpu.l = 0xAD;

        cpu.pc = 0;
        cpu.ram[0] = $opcode;
        let cycles = cpu.step();

        expect_eq!(cpu.ram[0xDEAD], 0xBA);
        expect_eq!(cpu.h, 0xDE);
        expect_eq!(cpu.l, 0xAE);
        expect_cycles!(cycles, 8);
      }
    };

    // LDI A,(HL)
    ($name: ident, $opcode: expr, a, (h l)) => {
      #[test]
      fn $name() {
        let mut cpu = Cpu::new();

        cpu.a = 0;
        cpu.ram[0xDEAD] = 0xBA;
        cpu.h = 0xDE;
        cpu.l = 0xAD;

        cpu.pc = 0;
        cpu.ram[0] = $opcode;
        let cycles = cpu.step();

        expect_eq!(cpu.a, 0xBA);
        expect_eq!(cpu.h, 0xDE);
        expect_eq!(cpu.l, 0xAE);
        expect_cycles!(cycles, 8);
      }
    };
  }

  macro_rules! test_ldd {

    // LDD (HL),A
    ($name: ident, $opcode: expr, (h l), a) => {
      #[test]
      fn $name() {
        let mut cpu = Cpu::new();

        cpu.a = 0xBA;
        cpu.ram[0xDEAD] = 0;
        cpu.h = 0xDE;
        cpu.l = 0xAD;

        cpu.pc = 0;
        cpu.ram[0] = $opcode;
        let cycles = cpu.step();

        expect_eq!(cpu.ram[0xDEAD], 0xBA);
        expect_eq!(cpu.h, 0xDE);
        expect_eq!(cpu.l, 0xAC);
        expect_cycles!(cycles, 8);
      }
    };

    // LDD A,(HL)
    ($name: ident, $opcode: expr, a, (h l)) => {
      #[test]
      fn $name() {
        let mut cpu = Cpu::new();

        cpu.a = 0;
        cpu.ram[0xDEAD] = 0xBA;
        cpu.h = 0xDE;
        cpu.l = 0xAD;

        cpu.pc = 0;
        cpu.ram[0] = $opcode;
        let cycles = cpu.step();

        expect_eq!(cpu.a, 0xBA);
        expect_eq!(cpu.h, 0xDE);
        expect_eq!(cpu.l, 0xAC);
        expect_cycles!(cycles, 8);
      }
    };
  }

  macro_rules! test_push {
    ($name:ident, $opcode:expr, [$rh:ident $rl:ident]) => {
      #[test]
      fn $name() {
        let mut cpu = Cpu::new();

        cpu.sp = 0xFFFE;
        cpu.ram[0xFFFC] = 0;
        cpu.ram[0xFFFD] = 0;
        cpu.$rh = 0xDE;
        cpu.$rl = 0xAD;

        cpu.pc = 0;
        cpu.ram[0] = $opcode;
        let cycles = cpu.step();

        expect_eq!(cpu.sp, 0xFFFC);
        expect_eq!(cpu.ram[0xFFFC], 0xAD);
        expect_eq!(cpu.ram[0xFFFD], 0xDE);
        expect_cycles!(cycles, 16);
      }
    };
  }

  macro_rules! test_pop {
    ($name:ident, $opcode:expr, [$rh:ident $rl:ident]) => {
      #[test]
      fn $name() {
        let mut cpu = Cpu::new();

        cpu.sp = 0xFFFC;
        cpu.ram[0xFFFC] = 0xAD;
        cpu.ram[0xFFFD] = 0xDE;
        cpu.$rh = 0;
        cpu.$rl = 0;

        cpu.pc = 0;
        cpu.ram[0] = $opcode;
        let cycles = cpu.step();

        expect_eq!(cpu.sp, 0xFFFE);
        expect_eq!(cpu.$rh, 0xDE);
        expect_eq!(cpu.$rl, 0xAD);
        expect_cycles!(cycles, 12);
      }
    };
  }

  macro_rules! test_add {

    // // ADD SP,dd
    // ($name:ident, $opcode:expr, dd) => {
    //   #[cfg(test)]
    //   mod $name {
    //     use super::super::*;

    //     #[test]
    //     fn add() {
    //       let mut cpu = Cpu::new();

    //       cpu.sp = 0x0001;

    //       cpu.pc = 0;
    //       cpu.ram[0] = $opcode;
    //       cpu.ram[1] = 0xFF;
    //       let cycles = cpu.step();

    //       expect_eq!(cpu.sp, 0x0402);
    //       expect_cycles!(cycles, 16);
    //       expect_flag!(z, cpu.n(), 0);
    //       expect_flag!(n, cpu.n(), 0);
    //       expect_flag!(h, cpu.h(), 0);
    //       expect_flag!(c, cpu.c(), 0);
    //     }

    //     #[test]
    //     fn wrap() {
    //       let mut cpu = Cpu::new();

    //       cpu.sp = 0xF000;
    //       cpu.ram[1] = 0x00;
    //       cpu.ram[2] = 0xF0;

    //       cpu.pc = 0;
    //       cpu.ram[0] = $opcode;
    //       let cycles = cpu.step();

    //       expect_eq!(cpu.sp, 0xE000);
    //       expect_cycles!(cycles, 16);
    //       expect_flag!(z, cpu.n(), 0);
    //       expect_flag!(n, cpu.n(), 0);
    //       expect_flag!(h, cpu.h(), 0);
    //       expect_flag!(c, cpu.c(), 1);
    //     }

    //     #[test]
    //     fn half_carry() {
    //       let mut cpu = Cpu::new();

    //       cpu.sp = 0x0F00;

    //       cpu.pc = 0;
    //       cpu.ram[0] = $opcode;
    //       cpu.ram[1] = 0x00;
    //       cpu.ram[2] = 0xF0;
    //       let cycles = cpu.step();

    //       expect_eq!(cpu.sp, 0x1E00);
    //       expect_cycles!(cycles, 16);
    //       expect_flag!(z, cpu.n(), 0);
    //       expect_flag!(n, cpu.n(), 0);
    //       expect_flag!(h, cpu.h(), 1);
    //       expect_flag!(c, cpu.c(), 0);
    //     }
    //   }
    // };

    // ADD HL,sp
    ($name:ident, $opcode:expr, sp) => {
      #[cfg(test)]
      mod $name {
        use super::super::*;

        #[test]
        fn add() {
          let mut cpu = Cpu::new();

          cpu.h = 2;
          cpu.l = 1;
          cpu.sp = 0x0201;

          cpu.pc = 0;
          cpu.ram[0] = $opcode;
          let cycles = cpu.step();

          expect_eq!(cpu.h, 4);
          expect_eq!(cpu.l, 2);
          expect_cycles!(cycles, 8);
          expect_flag!(n, cpu.n(), 0);
          expect_flag!(h, cpu.h(), 0);
          expect_flag!(c, cpu.c(), 0);
        }

        #[test]
        fn wrap() {
          let mut cpu = Cpu::new();

          cpu.h = 0xF0;
          cpu.l = 0x00;
          cpu.sp = 0xF000;

          cpu.pc = 0;
          cpu.ram[0] = $opcode;
          let cycles = cpu.step();

          expect_eq!(cpu.h, 0xE0);
          expect_eq!(cpu.l, 0x00);
          expect_cycles!(cycles, 8);
          expect_flag!(n, cpu.n(), 0);
          expect_flag!(h, cpu.h(), 0);
          expect_flag!(c, cpu.c(), 1);
        }

        #[test]
        fn half_carry() {
          let mut cpu = Cpu::new();

          cpu.h = 0x0F;
          cpu.l = 0x00;
          cpu.sp = 0x0F00;

          cpu.pc = 0;
          cpu.ram[0] = $opcode;
          let cycles = cpu.step();

          expect_eq!(cpu.h, 0x1E);
          expect_eq!(cpu.l, 0x00);
          expect_cycles!(cycles, 8);
          expect_flag!(n, cpu.n(), 0);
          expect_flag!(h, cpu.h(), 1);
          expect_flag!(c, cpu.c(), 0);
        }
      }
    };

    // ADD HL,rr
    ($name:ident, $opcode:expr, [$rh:ident $rl:ident]) => {
      #[cfg(test)]
      mod $name {
        use super::super::*;

        #[test]
        fn add() {
          let mut cpu = Cpu::new();

          cpu.h = 2;
          cpu.l = 1;
          cpu.$rh = 2;
          cpu.$rl = 1;

          cpu.pc = 0;
          cpu.ram[0] = $opcode;
          let cycles = cpu.step();

          expect_eq!(cpu.h, 4);
          expect_eq!(cpu.l, 2);
          expect_cycles!(cycles, 8);
          expect_flag!(n, cpu.n(), 0);
          expect_flag!(h, cpu.h(), 0);
          expect_flag!(c, cpu.c(), 0);
        }

        #[test]
        fn wrap() {
          let mut cpu = Cpu::new();

          cpu.h = 0xF0;
          cpu.l = 0x00;
          cpu.$rh = 0xF0;
          cpu.$rl = 0x00;

          cpu.pc = 0;
          cpu.ram[0] = $opcode;
          let cycles = cpu.step();

          expect_eq!(cpu.h, 0xE0);
          expect_eq!(cpu.l, 0x00);
          expect_cycles!(cycles, 8);
          expect_flag!(n, cpu.n(), 0);
          expect_flag!(h, cpu.h(), 0);
          expect_flag!(c, cpu.c(), 1);
        }

        #[test]
        fn half_carry() {
          let mut cpu = Cpu::new();

          cpu.h = 0x0F;
          cpu.l = 0x00;
          cpu.$rh = 0x0F;
          cpu.$rl = 0x00;

          cpu.pc = 0;
          cpu.ram[0] = $opcode;
          let cycles = cpu.step();

          expect_eq!(cpu.h, 0x1E);
          expect_eq!(cpu.l, 0x00);
          expect_cycles!(cycles, 8);
          expect_flag!(n, cpu.n(), 0);
          expect_flag!(h, cpu.h(), 1);
          expect_flag!(c, cpu.c(), 0);
        }
      }
    };

    // ADD A,(hl)
    ($name:ident, $opcode:expr, hl) => {
      #[cfg(test)]
      mod $name {
        use super::super::*;

        #[test]
        fn add() {
          let mut cpu = Cpu::new();

          cpu.a = 1;
          cpu.h = 0xDE;
          cpu.l = 0xAD;

          cpu.pc = 0;
          cpu.ram[0] = $opcode;
          cpu.ram[0xDEAD] = 1;
          let cycles = cpu.step();

          expect_eq!(cpu.a, 2);
          expect_cycles!(cycles, 8);
          expect_flag!(z, cpu.z(), 0);
          expect_flag!(n, cpu.n(), 0);
          expect_flag!(h, cpu.h(), 0);
          expect_flag!(c, cpu.c(), 0);
        }

        #[test]
        fn wrap() {
          let mut cpu = Cpu::new();

          cpu.a = 0x80;
          cpu.h = 0xDE;
          cpu.l = 0xAD;

          cpu.pc = 0;
          cpu.ram[0] = $opcode;
          cpu.ram[0xDEAD] = 0x80;
          let cycles = cpu.step();

          expect_eq!(cpu.a, 0);
          expect_cycles!(cycles, 8);
          expect_flag!(z, cpu.z(), 1);
          expect_flag!(n, cpu.n(), 0);
          expect_flag!(h, cpu.h(), 0);
          expect_flag!(c, cpu.c(), 1);
        }

        #[test]
        fn half_carry() {
          let mut cpu = Cpu::new();

          cpu.a = 0x0F;
          cpu.h = 0xDE;
          cpu.l = 0xAD;

          cpu.pc = 0;
          cpu.ram[0] = $opcode;
          cpu.ram[0xDEAD] = 0x0F;
          let cycles = cpu.step();

          expect_eq!(cpu.a, 0x1E);
          expect_cycles!(cycles, 8);
          expect_flag!(z, cpu.z(), 0);
          expect_flag!(n, cpu.n(), 0);
          expect_flag!(h, cpu.h(), 1);
          expect_flag!(c, cpu.c(), 0);
        }
      }
    };

    // ADD A,n
    ($name:ident, $opcode:expr, n) => {
      #[cfg(test)]
      mod $name {
        use super::super::*;

        #[test]
        fn add() {
          let mut cpu = Cpu::new();

          cpu.a = 1;

          cpu.pc = 0;
          cpu.ram[0] = $opcode;
          cpu.ram[1] = 1;
          let cycles = cpu.step();

          expect_eq!(cpu.a, 2);
          expect_cycles!(cycles, 8);
          expect_flag!(z, cpu.z(), 0);
          expect_flag!(n, cpu.n(), 0);
          expect_flag!(h, cpu.h(), 0);
          expect_flag!(c, cpu.c(), 0);
        }

        #[test]
        fn wrap() {
          let mut cpu = Cpu::new();

          cpu.a = 0x80;

          cpu.pc = 0;
          cpu.ram[0] = $opcode;
          cpu.ram[1] = 0x80;
          let cycles = cpu.step();

          expect_eq!(cpu.a, 0);
          expect_cycles!(cycles, 8);
          expect_flag!(z, cpu.z(), 1);
          expect_flag!(n, cpu.n(), 0);
          expect_flag!(h, cpu.h(), 0);
          expect_flag!(c, cpu.c(), 1);
        }

        #[test]
        fn half_carry() {
          let mut cpu = Cpu::new();

          cpu.a = 0x0F;

          cpu.pc = 0;
          cpu.ram[0] = $opcode;
          cpu.ram[1] = 0x0F;
          let cycles = cpu.step();

          expect_eq!(cpu.a, 0x1E);
          expect_cycles!(cycles, 8);
          expect_flag!(z, cpu.z(), 0);
          expect_flag!(n, cpu.n(), 0);
          expect_flag!(h, cpu.h(), 1);
          expect_flag!(c, cpu.c(), 0);
        }
      }
    };

    // ADD A,r
    ($name:ident, $opcode:expr, $r:ident) => {
      #[cfg(test)]
      mod $name {
        use super::super::*;

        #[test]
        fn add() {
          let mut cpu = Cpu::new();

          cpu.a = 1;
          cpu.$r = 1;

          cpu.pc = 0;
          cpu.ram[0] = $opcode;
          let cycles = cpu.step();

          expect_eq!(cpu.a, 2);
          expect_cycles!(cycles, 4);
          expect_flag!(z, cpu.z(), 0);
          expect_flag!(n, cpu.n(), 0);
          expect_flag!(h, cpu.h(), 0);
          expect_flag!(c, cpu.c(), 0);
        }

        #[test]
        fn wrap() {
          let mut cpu = Cpu::new();

          cpu.a = 0x80;
          cpu.$r = 0x80;

          cpu.pc = 0;
          cpu.ram[0] = $opcode;
          let cycles = cpu.step();

          expect_eq!(cpu.a, 0);
          expect_cycles!(cycles, 4);
          expect_flag!(z, cpu.z(), 1);
          expect_flag!(n, cpu.n(), 0);
          expect_flag!(h, cpu.h(), 0);
          expect_flag!(c, cpu.c(), 1);
        }

        #[test]
        fn half_carry() {
          let mut cpu = Cpu::new();

          cpu.a = 0x0F;
          cpu.$r = 0x0F;

          cpu.pc = 0;
          cpu.ram[0] = $opcode;
          let cycles = cpu.step();

          expect_eq!(cpu.a, 0x1E);
          expect_cycles!(cycles, 4);
          expect_flag!(z, cpu.z(), 0);
          expect_flag!(n, cpu.n(), 0);
          expect_flag!(h, cpu.h(), 1);
          expect_flag!(c, cpu.c(), 0);
        }
      }
    };

  }

  macro_rules! test_adc {

    // ADC A,(hl)
    ($name:ident, $opcode:expr, hl) => {
      #[cfg(test)]
      mod $name {
        use super::super::*;

        #[test]
        fn add() {
          let mut cpu = Cpu::new();

          cpu.a = 1;
          cpu.h = 0xDE;
          cpu.l = 0xAD;
          cpu.set_c();

          cpu.pc = 0;
          cpu.ram[0] = $opcode;
          cpu.ram[0xDEAD] = 1;
          let cycles = cpu.step();

          expect_eq!(cpu.a, 3);
          expect_cycles!(cycles, 8);
          expect_flag!(z, cpu.z(), 0);
          expect_flag!(n, cpu.n(), 0);
          expect_flag!(h, cpu.h(), 0);
          expect_flag!(c, cpu.c(), 0);
        }

        #[test]
        fn wrap() {
          let mut cpu = Cpu::new();

          cpu.a = 0x7F;
          cpu.h = 0xDE;
          cpu.l = 0xAD;
          cpu.set_c();

          cpu.pc = 0;
          cpu.ram[0] = $opcode;
          cpu.ram[0xDEAD] = 0x80;
          let cycles = cpu.step();

          expect_eq!(cpu.a, 0);
          expect_cycles!(cycles, 8);
          expect_flag!(z, cpu.z(), 1);
          expect_flag!(n, cpu.n(), 0);
          expect_flag!(h, cpu.h(), 1);
          expect_flag!(c, cpu.c(), 1);
        }

        #[test]
        fn half_carry() {
          let mut cpu = Cpu::new();

          cpu.a = 0x0F;
          cpu.h = 0xDE;
          cpu.l = 0xAD;
          cpu.set_c();

          cpu.pc = 0;
          cpu.ram[0] = $opcode;
          cpu.ram[0xDEAD] = 0x0F;
          let cycles = cpu.step();

          expect_eq!(cpu.a, 0x1F);
          expect_cycles!(cycles, 8);
          expect_flag!(z, cpu.z(), 0);
          expect_flag!(n, cpu.n(), 0);
          expect_flag!(h, cpu.h(), 1);
          expect_flag!(c, cpu.c(), 0);
        }
      }
    };

    // ADC A,n
    ($name:ident, $opcode:expr, n) => {
      #[cfg(test)]
      mod $name {
        use super::super::*;

        #[test]
        fn add() {
          let mut cpu = Cpu::new();

          cpu.a = 1;
          cpu.set_c();

          cpu.pc = 0;
          cpu.ram[0] = $opcode;
          cpu.ram[1] = 1;
          let cycles = cpu.step();

          expect_eq!(cpu.a, 3);
          expect_cycles!(cycles, 8);
          expect_flag!(z, cpu.z(), 0);
          expect_flag!(n, cpu.n(), 0);
          expect_flag!(h, cpu.h(), 0);
          expect_flag!(c, cpu.c(), 0);
        }

        #[test]
        fn wrap() {
          let mut cpu = Cpu::new();

          cpu.a = 0x7F;
          cpu.set_c();

          cpu.pc = 0;
          cpu.ram[0] = $opcode;
          cpu.ram[1] = 0x80;
          let cycles = cpu.step();

          expect_eq!(cpu.a, 0);
          expect_cycles!(cycles, 8);
          expect_flag!(z, cpu.z(), 1);
          expect_flag!(n, cpu.n(), 0);
          expect_flag!(h, cpu.h(), 1);
          expect_flag!(c, cpu.c(), 1);
        }

        #[test]
        fn half_carry() {
          let mut cpu = Cpu::new();

          cpu.a = 0x0F;
          cpu.set_c();

          cpu.pc = 0;
          cpu.ram[0] = $opcode;
          cpu.ram[1] = 0x0F;
          let cycles = cpu.step();

          expect_eq!(cpu.a, 0x1F);
          expect_cycles!(cycles, 8);
          expect_flag!(z, cpu.z(), 0);
          expect_flag!(n, cpu.n(), 0);
          expect_flag!(h, cpu.h(), 1);
          expect_flag!(c, cpu.c(), 0);
        }
      }
    };

    // ADC A,r
    ($name:ident, $opcode:expr, $r:ident) => {
      #[cfg(test)]
      mod $name {
        use super::super::*;

        #[test]
        fn add() {
          let mut cpu = Cpu::new();

          cpu.a = 1;
          cpu.$r = 1;
          cpu.set_c();

          cpu.pc = 0;
          cpu.ram[0] = $opcode;
          let cycles = cpu.step();

          expect_eq!(cpu.a, 3);
          expect_cycles!(cycles, 4);
          expect_flag!(z, cpu.z(), 0);
          expect_flag!(n, cpu.n(), 0);
          expect_flag!(h, cpu.h(), 0);
          expect_flag!(c, cpu.c(), 0);
        }

        #[test]
        fn wrap() {
          let mut cpu = Cpu::new();

          cpu.a = 0x80;
          cpu.$r = 0x80;
          cpu.set_c();

          cpu.pc = 0;
          cpu.ram[0] = $opcode;
          let cycles = cpu.step();

          expect_eq!(cpu.a, 1);
          expect_cycles!(cycles, 4);
          expect_flag!(z, cpu.z(), 0);
          expect_flag!(n, cpu.n(), 0);
          expect_flag!(h, cpu.h(), 0);
          expect_flag!(c, cpu.c(), 1);
        }

        #[test]
        fn half_carry() {
          let mut cpu = Cpu::new();

          cpu.a = 0x0F;
          cpu.$r = 0x0F;
          cpu.set_c();

          cpu.pc = 0;
          cpu.ram[0] = $opcode;
          let cycles = cpu.step();

          expect_eq!(cpu.a, 0x1F);
          expect_cycles!(cycles, 4);
          expect_flag!(z, cpu.z(), 0);
          expect_flag!(n, cpu.n(), 0);
          expect_flag!(h, cpu.h(), 1);
          expect_flag!(c, cpu.c(), 0);
        }
      }
    };

  }

  macro_rules! test_sub {

    // SUB A,(hl)
    ($name:ident, $opcode:expr, hl) => {
      #[cfg(test)]
      mod $name {
        use super::super::*;

        #[test]
        fn add() {
          let mut cpu = Cpu::new();

          cpu.a = 1;
          cpu.h = 0xDE;
          cpu.l = 0xAD;

          cpu.pc = 0;
          cpu.ram[0] = $opcode;
          cpu.ram[0xDEAD] = 1;
          let cycles = cpu.step();

          expect_eq!(cpu.a, 0);
          expect_cycles!(cycles, 8);
          expect_flag!(z, cpu.z(), 1);
          expect_flag!(n, cpu.n(), 1);
          expect_flag!(h, cpu.h(), 0);
          expect_flag!(c, cpu.c(), 0);
        }

        #[test]
        fn wrap() {
          let mut cpu = Cpu::new();

          cpu.a = 0x80;
          cpu.h = 0xDE;
          cpu.l = 0xAD;

          cpu.pc = 0;
          cpu.ram[0] = $opcode;
          cpu.ram[0xDEAD] = 0x90;
          let cycles = cpu.step();

          expect_eq!(cpu.a, 0xF0);
          expect_cycles!(cycles, 8);
          expect_flag!(z, cpu.z(), 0);
          expect_flag!(n, cpu.n(), 1);
          expect_flag!(h, cpu.h(), 0);
          expect_flag!(c, cpu.c(), 1);
        }

        #[test]
        fn half_carry() {
          let mut cpu = Cpu::new();

          cpu.a = 0x20;
          cpu.h = 0xDE;
          cpu.l = 0xAD;

          cpu.pc = 0;
          cpu.ram[0] = $opcode;
          cpu.ram[0xDEAD] = 0x1F;
          let cycles = cpu.step();

          expect_eq!(cpu.a, 0x01);
          expect_cycles!(cycles, 8);
          expect_flag!(z, cpu.z(), 0);
          expect_flag!(n, cpu.n(), 1);
          expect_flag!(h, cpu.h(), 1);
          expect_flag!(c, cpu.c(), 0);
        }
      }
    };

    // SUB A,n
    ($name:ident, $opcode:expr, n) => {
      #[cfg(test)]
      mod $name {
        use super::super::*;

        #[test]
        fn add() {
          let mut cpu = Cpu::new();

          cpu.a = 1;

          cpu.pc = 0;
          cpu.ram[0] = $opcode;
          cpu.ram[1] = 1;
          let cycles = cpu.step();

          expect_eq!(cpu.a, 0);
          expect_cycles!(cycles, 8);
          expect_flag!(z, cpu.z(), 1);
          expect_flag!(n, cpu.n(), 1);
          expect_flag!(h, cpu.h(), 0);
          expect_flag!(c, cpu.c(), 0);
        }

        #[test]
        fn wrap() {
          let mut cpu = Cpu::new();

          cpu.a = 0x80;

          cpu.pc = 0;
          cpu.ram[0] = $opcode;
          cpu.ram[1] = 0x90;
          let cycles = cpu.step();

          expect_eq!(cpu.a, 0xF0);
          expect_cycles!(cycles, 8);
          expect_flag!(z, cpu.z(), 0);
          expect_flag!(n, cpu.n(), 1);
          expect_flag!(h, cpu.h(), 0);
          expect_flag!(c, cpu.c(), 1);
        }

        #[test]
        fn half_carry() {
          let mut cpu = Cpu::new();

          cpu.a = 0x20;

          cpu.pc = 0;
          cpu.ram[0] = $opcode;
          cpu.ram[1] = 0x1F;
          let cycles = cpu.step();

          expect_eq!(cpu.a, 0x01);
          expect_cycles!(cycles, 8);
          expect_flag!(z, cpu.z(), 0);
          expect_flag!(n, cpu.n(), 1);
          expect_flag!(h, cpu.h(), 1);
          expect_flag!(c, cpu.c(), 0);
        }
      }
    };

    // SUB A,A
    ($name:ident, $opcode:expr, a) => {
      #[cfg(test)]
      mod $name {
        use super::super::*;

        #[test]
        fn sub() {
          let mut cpu = Cpu::new();

          cpu.a = 1;

          cpu.pc = 0;
          cpu.ram[0] = $opcode;
          let cycles = cpu.step();

          expect_eq!(cpu.a, 0);
          expect_cycles!(cycles, 4);
          expect_flag!(z, cpu.z(), 1);
          expect_flag!(n, cpu.n(), 1);
          expect_flag!(h, cpu.h(), 0);
          expect_flag!(c, cpu.c(), 0);
        }
      }
    };

    // SUB A,r
    ($name:ident, $opcode:expr, $r:ident) => {
      #[cfg(test)]
      mod $name {
        use super::super::*;

        #[test]
        fn sub() {
          let mut cpu = Cpu::new();

          cpu.a = 1;
          cpu.$r = 1;

          cpu.pc = 0;
          cpu.ram[0] = $opcode;
          let cycles = cpu.step();

          expect_eq!(cpu.a, 0);
          expect_cycles!(cycles, 4);
          expect_flag!(z, cpu.z(), 1);
          expect_flag!(n, cpu.n(), 1);
          expect_flag!(h, cpu.h(), 0);
          expect_flag!(c, cpu.c(), 0);
        }

        #[test]
        fn wrap() {
          let mut cpu = Cpu::new();

          cpu.a = 0x80;
          cpu.$r = 0x90;

          cpu.pc = 0;
          cpu.ram[0] = $opcode;
          let cycles = cpu.step();

          expect_eq!(cpu.a, 0xF0);
          expect_cycles!(cycles, 4);
          expect_flag!(z, cpu.z(), 0);
          expect_flag!(n, cpu.n(), 1);
          expect_flag!(h, cpu.h(), 0);
          expect_flag!(c, cpu.c(), 1);
        }

        #[test]
        fn half_carry() {
          let mut cpu = Cpu::new();

          cpu.a = 0x20;
          cpu.$r = 0x1F;

          cpu.pc = 0;
          cpu.ram[0] = $opcode;
          let cycles = cpu.step();

          expect_eq!(cpu.a, 0x01);
          expect_cycles!(cycles, 4);
          expect_flag!(z, cpu.z(), 0);
          expect_flag!(n, cpu.n(), 1);
          expect_flag!(h, cpu.h(), 1);
          expect_flag!(c, cpu.c(), 0);
        }
      }
    };

  }

  macro_rules! test_sbc {

    // SBC A,(hl)
    ($name:ident, $opcode:expr, hl) => {
      #[cfg(test)]
      mod $name {
        use super::super::*;

        #[test]
        fn add() {
          let mut cpu = Cpu::new();

          cpu.a = 2;
          cpu.h = 0xDE;
          cpu.l = 0xAD;
          cpu.set_c();

          cpu.pc = 0;
          cpu.ram[0] = $opcode;
          cpu.ram[0xDEAD] = 1;
          let cycles = cpu.step();

          expect_eq!(cpu.a, 0);
          expect_cycles!(cycles, 8);
          expect_flag!(z, cpu.z(), 1);
          expect_flag!(n, cpu.n(), 1);
          expect_flag!(h, cpu.h(), 0);
          expect_flag!(c, cpu.c(), 0);
        }

        #[test]
        fn wrap() {
          let mut cpu = Cpu::new();

          cpu.a = 0x81;
          cpu.h = 0xDE;
          cpu.l = 0xAD;
          cpu.set_c();

          cpu.pc = 0;
          cpu.ram[0] = $opcode;
          cpu.ram[0xDEAD] = 0x90;
          let cycles = cpu.step();

          expect_eq!(cpu.a, 0xF0);
          expect_cycles!(cycles, 8);
          expect_flag!(z, cpu.z(), 0);
          expect_flag!(n, cpu.n(), 1);
          expect_flag!(h, cpu.h(), 0);
          expect_flag!(c, cpu.c(), 1);
        }

        #[test]
        fn half_carry() {
          let mut cpu = Cpu::new();

          cpu.a = 0x21;
          cpu.h = 0xDE;
          cpu.l = 0xAD;
          cpu.set_c();

          cpu.pc = 0;
          cpu.ram[0] = $opcode;
          cpu.ram[0xDEAD] = 0x1F;
          let cycles = cpu.step();

          expect_eq!(cpu.a, 0x01);
          expect_cycles!(cycles, 8);
          expect_flag!(z, cpu.z(), 0);
          expect_flag!(n, cpu.n(), 1);
          expect_flag!(h, cpu.h(), 1);
          expect_flag!(c, cpu.c(), 0);
        }
      }
    };

    // SBC A,n
    ($name:ident, $opcode:expr, n) => {
      #[cfg(test)]
      mod $name {
        use super::super::*;

        #[test]
        fn add() {
          let mut cpu = Cpu::new();

          cpu.a = 2;
          cpu.set_c();

          cpu.pc = 0;
          cpu.ram[0] = $opcode;
          cpu.ram[1] = 1;
          let cycles = cpu.step();

          expect_eq!(cpu.a, 0);
          expect_cycles!(cycles, 8);
          expect_flag!(z, cpu.z(), 1);
          expect_flag!(n, cpu.n(), 1);
          expect_flag!(h, cpu.h(), 0);
          expect_flag!(c, cpu.c(), 0);
        }

        #[test]
        fn wrap() {
          let mut cpu = Cpu::new();

          cpu.a = 0x81;
          cpu.set_c();

          cpu.pc = 0;
          cpu.ram[0] = $opcode;
          cpu.ram[1] = 0x90;
          let cycles = cpu.step();

          expect_eq!(cpu.a, 0xF0);
          expect_cycles!(cycles, 8);
          expect_flag!(z, cpu.z(), 0);
          expect_flag!(n, cpu.n(), 1);
          expect_flag!(h, cpu.h(), 0);
          expect_flag!(c, cpu.c(), 1);
        }

        #[test]
        fn half_carry() {
          let mut cpu = Cpu::new();

          cpu.a = 0x21;
          cpu.set_c();

          cpu.pc = 0;
          cpu.ram[0] = $opcode;
          cpu.ram[1] = 0x1F;
          let cycles = cpu.step();

          expect_eq!(cpu.a, 0x01);
          expect_cycles!(cycles, 8);
          expect_flag!(z, cpu.z(), 0);
          expect_flag!(n, cpu.n(), 1);
          expect_flag!(h, cpu.h(), 1);
          expect_flag!(c, cpu.c(), 0);
        }
      }
    };

    // SBC A,A
    ($name:ident, $opcode:expr, a) => {
      #[cfg(test)]
      mod $name {
        use super::super::*;

        #[test]
        fn sub() {
          let mut cpu = Cpu::new();

          cpu.a = 1;
          cpu.set_c();

          cpu.pc = 0;
          cpu.ram[0] = $opcode;
          let cycles = cpu.step();

          expect_eq!(cpu.a, 0xFF);
          expect_cycles!(cycles, 4);
          expect_flag!(z, cpu.z(), 0);
          expect_flag!(n, cpu.n(), 1);
          expect_flag!(h, cpu.h(), 1);
          expect_flag!(c, cpu.c(), 1);
        }
      }
    };

    // SBC A,r
    ($name:ident, $opcode:expr, $r:ident) => {
      #[cfg(test)]
      mod $name {
        use super::super::*;

        #[test]
        fn sub() {
          let mut cpu = Cpu::new();

          cpu.a = 2;
          cpu.$r = 1;
          cpu.set_c();

          cpu.pc = 0;
          cpu.ram[0] = $opcode;
          let cycles = cpu.step();

          expect_eq!(cpu.a, 0);
          expect_cycles!(cycles, 4);
          expect_flag!(z, cpu.z(), 1);
          expect_flag!(n, cpu.n(), 1);
          expect_flag!(h, cpu.h(), 0);
          expect_flag!(c, cpu.c(), 0);
        }

        #[test]
        fn wrap() {
          let mut cpu = Cpu::new();

          cpu.a = 0x81;
          cpu.$r = 0x90;
          cpu.set_c();

          cpu.pc = 0;
          cpu.ram[0] = $opcode;
          let cycles = cpu.step();

          expect_eq!(cpu.a, 0xF0);
          expect_cycles!(cycles, 4);
          expect_flag!(z, cpu.z(), 0);
          expect_flag!(n, cpu.n(), 1);
          expect_flag!(h, cpu.h(), 0);
          expect_flag!(c, cpu.c(), 1);
        }

        #[test]
        fn half_carry() {
          let mut cpu = Cpu::new();

          cpu.a = 0x21;
          cpu.$r = 0x1F;
          cpu.set_c();

          cpu.pc = 0;
          cpu.ram[0] = $opcode;
          let cycles = cpu.step();

          expect_eq!(cpu.a, 0x01);
          expect_cycles!(cycles, 4);
          expect_flag!(z, cpu.z(), 0);
          expect_flag!(n, cpu.n(), 1);
          expect_flag!(h, cpu.h(), 1);
          expect_flag!(c, cpu.c(), 0);
        }
      }
    };

  }

  macro_rules! test_and {

    // AND A,(HL)
    ($name:ident, $opcode:expr, hl) => {
      #[test]
      fn $name() {
        let mut cpu = Cpu::new();

        cpu.a = 0xFF;
        cpu.h = 0xDE;
        cpu.l = 0xAD;

        cpu.pc = 0;
        cpu.ram[0] = $opcode;
        cpu.ram[0xDEAD] = 0xFF;
        let cycles = cpu.step();

        expect_eq!(cpu.a, 0xFF);
        expect_cycles!(cycles, 8);
        expect_flag!(z, cpu.z(), 0);
        expect_flag!(n, cpu.n(), 0);
        expect_flag!(h, cpu.h(), 1);
        expect_flag!(c, cpu.c(), 0);
      }
    };

    // AND A,n
    ($name:ident, $opcode:expr, n) => {
      #[test]
      fn $name() {
        let mut cpu = Cpu::new();

        cpu.a = 0xFF;

        cpu.pc = 0;
        cpu.ram[0] = $opcode;
        cpu.ram[1] = 0xFF;
        let cycles = cpu.step();

        expect_eq!(cpu.a, 0xFF);
        expect_cycles!(cycles, 8);
        expect_flag!(z, cpu.z(), 0);
        expect_flag!(n, cpu.n(), 0);
        expect_flag!(h, cpu.h(), 1);
        expect_flag!(c, cpu.c(), 0);
      }
    };

    // AND A,r
    ($name:ident, $opcode:expr, $r:ident) => {
      #[test]
      fn $name() {
        let mut cpu = Cpu::new();

        cpu.a = 0xFF;
        cpu.$r = 0xFF;

        cpu.pc = 0;
        cpu.ram[0] = $opcode;
        let cycles = cpu.step();

        expect_eq!(cpu.a, 0xFF);
        expect_cycles!(cycles, 4);
        expect_flag!(z, cpu.z(), 0);
        expect_flag!(n, cpu.n(), 0);
        expect_flag!(h, cpu.h(), 1);
        expect_flag!(c, cpu.c(), 0);
      }
    };
  }

  macro_rules! test_xor {

    // XOR A,(HL)
    ($name:ident, $opcode:expr, hl) => {
      #[test]
      fn $name() {
        let mut cpu = Cpu::new();

        cpu.a = 0xF0;
        cpu.h = 0xDE;
        cpu.l = 0xAD;

        cpu.pc = 0;
        cpu.ram[0] = $opcode;
        cpu.ram[0xDEAD] = 0x0F;
        let cycles = cpu.step();

        expect_eq!(cpu.a, 0xFF);
        expect_cycles!(cycles, 8);
        expect_flag!(z, cpu.z(), 0);
        expect_flag!(n, cpu.n(), 0);
        expect_flag!(h, cpu.h(), 0);
        expect_flag!(c, cpu.c(), 0);
      }
    };

    // XOR A,n
    ($name:ident, $opcode:expr, n) => {
      #[test]
      fn $name() {
        let mut cpu = Cpu::new();

        cpu.a = 0xF0;

        cpu.pc = 0;
        cpu.ram[0] = $opcode;
        cpu.ram[1] = 0x0F;
        let cycles = cpu.step();

        expect_eq!(cpu.a, 0xFF);
        expect_cycles!(cycles, 8);
        expect_flag!(z, cpu.z(), 0);
        expect_flag!(n, cpu.n(), 0);
        expect_flag!(h, cpu.h(), 0);
        expect_flag!(c, cpu.c(), 0);
      }
    };

    // XOR A,r
    ($name:ident, $opcode:expr, $r:ident) => {
      #[test]
      fn $name() {
        let mut cpu = Cpu::new();

        cpu.a = 0xFF;
        cpu.$r = 0xFF;

        cpu.pc = 0;
        cpu.ram[0] = $opcode;
        let cycles = cpu.step();

        expect_eq!(cpu.a, 0);
        expect_cycles!(cycles, 4);
        expect_flag!(z, cpu.z(), 1);
        expect_flag!(n, cpu.n(), 0);
        expect_flag!(h, cpu.h(), 0);
        expect_flag!(c, cpu.c(), 0);
      }
    };
  }

  macro_rules! test_or {

    // OR A,(HL)
    ($name:ident, $opcode:expr, hl) => {
      #[test]
      fn $name() {
        let mut cpu = Cpu::new();

        cpu.a = 0xF0;
        cpu.h = 0xDE;
        cpu.l = 0xAD;

        cpu.pc = 0;
        cpu.ram[0] = $opcode;
        cpu.ram[0xDEAD] = 0x0F;
        let cycles = cpu.step();

        expect_eq!(cpu.a, 0xFF);
        expect_cycles!(cycles, 8);
        expect_flag!(z, cpu.z(), 0);
        expect_flag!(n, cpu.n(), 0);
        expect_flag!(h, cpu.h(), 0);
        expect_flag!(c, cpu.c(), 0);
      }
    };

    // OR A,n
    ($name:ident, $opcode:expr, n) => {
      #[test]
      fn $name() {
        let mut cpu = Cpu::new();

        cpu.a = 0;

        cpu.pc = 0;
        cpu.ram[0] = $opcode;
        cpu.ram[1] = 0;
        let cycles = cpu.step();

        expect_eq!(cpu.a, 0);
        expect_cycles!(cycles, 8);
        expect_flag!(z, cpu.z(), 1);
        expect_flag!(n, cpu.n(), 0);
        expect_flag!(h, cpu.h(), 0);
        expect_flag!(c, cpu.c(), 0);
      }
    };

    // OR A,r
    ($name:ident, $opcode:expr, $r:ident) => {
      #[test]
      fn $name() {
        let mut cpu = Cpu::new();

        cpu.a = 0x0F;
        cpu.$r = 0x0F;

        cpu.pc = 0;
        cpu.ram[0] = $opcode;
        let cycles = cpu.step();

        expect_eq!(cpu.a, 0xF);
        expect_cycles!(cycles, 4);
        expect_flag!(z, cpu.z(), 0);
        expect_flag!(n, cpu.n(), 0);
        expect_flag!(h, cpu.h(), 0);
        expect_flag!(c, cpu.c(), 0);
      }
    };
  }

  macro_rules! test_cp {

    // CP A,(HL)
    ($name:ident, $opcode:expr, hl) => {
      #[test]
      fn $name() {
        let mut cpu = Cpu::new();

        cpu.a = 0xF0;
        cpu.h = 0xDE;
        cpu.l = 0xAD;

        cpu.pc = 0;
        cpu.ram[0] = $opcode;
        cpu.ram[0xDEAD] = 0x0F;
        let cycles = cpu.step();

        expect_cycles!(cycles, 8);
        expect_flag!(z, cpu.z(), 0);
        expect_flag!(n, cpu.n(), 1);
        expect_flag!(h, cpu.h(), 1);
        expect_flag!(c, cpu.c(), 0);
      }
    };

    // CP A,n
    ($name:ident, $opcode:expr, n) => {
      #[test]
      fn $name() {
        let mut cpu = Cpu::new();

        cpu.a = 0;

        cpu.pc = 0;
        cpu.ram[0] = $opcode;
        cpu.ram[1] = 0x0F;
        let cycles = cpu.step();

        expect_cycles!(cycles, 8);
        expect_flag!(z, cpu.z(), 0);
        expect_flag!(n, cpu.n(), 1);
        expect_flag!(h, cpu.h(), 1);
        expect_flag!(c, cpu.c(), 1);
      }
    };

    // CP A,r
    ($name:ident, $opcode:expr, $r:ident) => {
      #[test]
      fn $name() {
        let mut cpu = Cpu::new();

        cpu.a = 0x0F;
        cpu.$r = 0x0F;

        cpu.pc = 0;
        cpu.ram[0] = $opcode;
        let cycles = cpu.step();

        expect_cycles!(cycles, 4);
        expect_flag!(z, cpu.z(), 1);
        expect_flag!(n, cpu.n(), 1);
        expect_flag!(h, cpu.h(), 0);
        expect_flag!(c, cpu.c(), 0);
      }
    };
  }

  macro_rules! test_inc {
    // INC rr
    ($name: ident, $opcode: expr, [$rh: ident $rl: ident]) => {
      #[cfg(test)]
      mod $name {
        use super::super::*;

        #[test]
        fn inc() {
          let mut cpu = Cpu::new();

          cpu.$rh = 0;
          cpu.$rl = 0;

          cpu.pc = 0;
          cpu.ram[0] = $opcode;
          let cycles = cpu.step();

          expect_eq!(cpu.$rh, 0);
          expect_eq!(cpu.$rl, 1);
          expect_cycles!(cycles, 8);
        }

        #[test]
        fn wrap() {
          let mut cpu = Cpu::new();

          cpu.$rh = 0xFF;
          cpu.$rl = 0xFF;

          cpu.pc = 0;
          cpu.ram[0] = $opcode;
          let cycles = cpu.step();

          expect_eq!(cpu.$rh, 0);
          expect_eq!(cpu.$rl, 0);
          expect_cycles!(cycles, 8);
        }
      }
    };

    // INC (hl)
    ($name: ident, $opcode: expr, hl) => {
      #[cfg(test)]
      mod $name {
        use super::super::*;

        #[test]
        fn inc() {
          let mut cpu = Cpu::new();

          cpu.ram[0xDEAD] = 0;
          cpu.h = 0xDE;
          cpu.l = 0xAD;

          cpu.pc = 0;
          cpu.ram[0] = $opcode;
          let cycles = cpu.step();

          expect_eq!(cpu.ram[0xDEAD], 1);
          expect_cycles!(cycles, 12);
          expect_flag!(z, cpu.z(), 0);
          expect_flag!(n, cpu.n(), 0);
          expect_flag!(h, cpu.h(), 0);
        }

        #[test]
        fn wrap() {
          let mut cpu = Cpu::new();

          cpu.ram[0xDEAD] = 0xFF;
          cpu.h = 0xDE;
          cpu.l = 0xAD;

          cpu.pc = 0;
          cpu.ram[0] = $opcode;
          let cycles = cpu.step();

          expect_eq!(cpu.ram[0xDEAD], 0);
          expect_cycles!(cycles, 12);
          expect_flag!(z, cpu.z(), 1);
          expect_flag!(n, cpu.n(), 0);
          expect_flag!(h, cpu.h(), 1);
        }
      }
    };

    // INC sp
    ($name: ident, $opcode: expr, sp) => {
      #[cfg(test)]
      mod $name {
        use super::super::*;

        #[test]
        fn inc() {
          let mut cpu = Cpu::new();

          cpu.sp = 0;

          cpu.pc = 0;
          cpu.ram[0] = $opcode;
          let cycles = cpu.step();

          expect_eq!(cpu.sp, 1);
          expect_cycles!(cycles, 8);
        }

        #[test]
        fn wrap() {
          let mut cpu = Cpu::new();

          cpu.sp = 0xFFFF;

          cpu.pc = 0;
          cpu.ram[0] = $opcode;
          let cycles = cpu.step();

          expect_eq!(cpu.sp, 0);
          expect_cycles!(cycles, 8);
        }
      }
    };

    // INC r
    ($name: ident, $opcode: expr, $r: ident) => {
      #[cfg(test)]
      mod $name {
        use super::super::*;

        #[test]
        fn inc() {
          let mut cpu = Cpu::new();

          cpu.$r = 0;

          cpu.pc = 0;
          cpu.ram[0] = $opcode;
          let cycles = cpu.step();

          expect_eq!(cpu.$r, 1);
          expect_cycles!(cycles, 4);
          expect_flag!(z, cpu.z(), 0);
          expect_flag!(n, cpu.n(), 0);
          expect_flag!(h, cpu.h(), 0);
        }

        #[test]
        fn wrap() {
          let mut cpu = Cpu::new();

          cpu.$r = 0xFF;

          cpu.pc = 0;
          cpu.ram[0] = $opcode;
          let cycles = cpu.step();

          expect_eq!(cpu.$r, 0);
          expect_cycles!(cycles, 4);
          expect_flag!(z, cpu.z(), 1);
          expect_flag!(n, cpu.n(), 0);
          expect_flag!(h, cpu.h(), 1);
        }
      }
    };

  }

  macro_rules! test_dec {
    // DEC rr
    ($name: ident, $opcode: expr, [$rh: ident $rl: ident]) => {
      #[cfg(test)]
      mod $name {
        use super::super::*;

        #[test]
        fn inc() {
          let mut cpu = Cpu::new();

          cpu.$rh = 0;
          cpu.$rl = 1;

          cpu.pc = 0;
          cpu.ram[0] = $opcode;
          let cycles = cpu.step();

          expect_eq!(cpu.$rh, 0);
          expect_eq!(cpu.$rl, 0);
          expect_cycles!(cycles, 8);
        }

        #[test]
        fn wrap() {
          let mut cpu = Cpu::new();

          cpu.$rh = 0;
          cpu.$rl = 0;

          cpu.pc = 0;
          cpu.ram[0] = $opcode;
          let cycles = cpu.step();

          expect_eq!(cpu.$rh, 0xFF);
          expect_eq!(cpu.$rl, 0xFF);
          expect_cycles!(cycles, 8);
        }
      }
    };

    // DEC (hl)
    ($name: ident, $opcode: expr, hl) => {
      #[cfg(test)]
      mod $name {
        use super::super::*;

        #[test]
        fn inc() {
          let mut cpu = Cpu::new();

          cpu.ram[0xDEAD] = 1;
          cpu.h = 0xDE;
          cpu.l = 0xAD;

          cpu.pc = 0;
          cpu.ram[0] = $opcode;
          let cycles = cpu.step();

          expect_eq!(cpu.ram[0xDEAD], 0);
          expect_cycles!(cycles, 12);
          expect_flag!(z, cpu.z(), 1);
          expect_flag!(n, cpu.n(), 1);
          expect_flag!(h, cpu.h(), 0);
        }

        #[test]
        fn wrap() {
          let mut cpu = Cpu::new();

          cpu.ram[0xDEAD] = 0;
          cpu.h = 0xDE;
          cpu.l = 0xAD;

          cpu.pc = 0;
          cpu.ram[0] = $opcode;
          let cycles = cpu.step();

          expect_eq!(cpu.ram[0xDEAD], 0xFF);
          expect_cycles!(cycles, 12);
          expect_flag!(z, cpu.z(), 0);
          expect_flag!(n, cpu.n(), 1);
          expect_flag!(h, cpu.h(), 1);
        }
      }
    };

    // DEC sp
    ($name: ident, $opcode: expr, sp) => {
      #[cfg(test)]
      mod $name {
        use super::super::*;

        #[test]
        fn inc() {
          let mut cpu = Cpu::new();

          cpu.sp = 1;

          cpu.pc = 0;
          cpu.ram[0] = $opcode;
          let cycles = cpu.step();

          expect_eq!(cpu.sp, 0);
          expect_cycles!(cycles, 8);
        }

        #[test]
        fn wrap() {
          let mut cpu = Cpu::new();

          cpu.sp = 0;

          cpu.pc = 0;
          cpu.ram[0] = $opcode;
          let cycles = cpu.step();

          expect_eq!(cpu.sp, 0xFFFF);
          expect_cycles!(cycles, 8);
        }
      }
    };

    // DEC r
    ($name: ident, $opcode: expr, $r: ident) => {
      #[cfg(test)]
      mod $name {
        use super::super::*;

        #[test]
        fn inc() {
          let mut cpu = Cpu::new();

          cpu.$r = 1;

          cpu.pc = 0;
          cpu.ram[0] = $opcode;
          let cycles = cpu.step();

          expect_eq!(cpu.$r, 0);
          expect_cycles!(cycles, 4);
          expect_flag!(z, cpu.z(), 1);
          expect_flag!(n, cpu.n(), 1);
          expect_flag!(h, cpu.h(), 0);
        }

        #[test]
        fn wrap() {
          let mut cpu = Cpu::new();

          cpu.$r = 0;

          cpu.pc = 0;
          cpu.ram[0] = $opcode;
          let cycles = cpu.step();

          expect_eq!(cpu.$r, 0xFF);
          expect_cycles!(cycles, 4);
          expect_flag!(z, cpu.z(), 0);
          expect_flag!(n, cpu.n(), 1);
          expect_flag!(h, cpu.h(), 1);
        }
      }
    };

  }

  #[test]
  fn nop() {
    let mut cpu = Cpu::new();

    cpu.pc = 0;
    cpu.ram[0] = 0x00;
    let cycles = cpu.step();

    assert_eq!(4, cycles);
  }

  test_ld!(ld_a_b, 0x78, a, b);
  test_ld!(ld_a_c, 0x79, a, c);
  test_ld!(ld_a_d, 0x7A, a, d);
  test_ld!(ld_a_e, 0x7B, a, e);
  test_ld!(ld_a_h, 0x7C, a, h);
  test_ld!(ld_a_l, 0x7D, a, l);
  test_ld!(ld_a_a, 0x7F, a, a);

  test_ld!(ld_b_b, 0x40, b, b);
  test_ld!(ld_b_c, 0x41, b, c);
  test_ld!(ld_b_d, 0x42, b, d);
  test_ld!(ld_b_e, 0x43, b, e);
  test_ld!(ld_b_h, 0x44, b, h);
  test_ld!(ld_b_l, 0x45, b, l);
  test_ld!(ld_b_a, 0x47, b, a);

  test_ld!(ld_c_b, 0x48, c, b);
  test_ld!(ld_c_c, 0x49, c, c);
  test_ld!(ld_c_d, 0x4A, c, d);
  test_ld!(ld_c_e, 0x4B, c, e);
  test_ld!(ld_c_h, 0x4C, c, h);
  test_ld!(ld_c_l, 0x4D, c, l);
  test_ld!(ld_c_a, 0x4F, c, a);

  test_ld!(ld_d_b, 0x50, d, b);
  test_ld!(ld_d_c, 0x51, d, c);
  test_ld!(ld_d_d, 0x52, d, d);
  test_ld!(ld_d_e, 0x53, d, e);
  test_ld!(ld_d_h, 0x54, d, h);
  test_ld!(ld_d_l, 0x55, d, l);
  test_ld!(ld_d_a, 0x57, d, a);

  test_ld!(ld_e_b, 0x58, e, b);
  test_ld!(ld_e_c, 0x59, e, c);
  test_ld!(ld_e_d, 0x5A, e, d);
  test_ld!(ld_e_e, 0x5B, e, e);
  test_ld!(ld_e_h, 0x5C, e, h);
  test_ld!(ld_e_l, 0x5D, e, l);
  test_ld!(ld_e_a, 0x5F, e, a);

  test_ld!(ld_h_b, 0x60, h, b);
  test_ld!(ld_h_c, 0x61, h, c);
  test_ld!(ld_h_d, 0x62, h, d);
  test_ld!(ld_h_e, 0x63, h, e);
  test_ld!(ld_h_h, 0x64, h, h);
  test_ld!(ld_h_l, 0x65, h, l);
  test_ld!(ld_h_a, 0x67, h, a);

  test_ld!(ld_l_b, 0x68, l, b);
  test_ld!(ld_l_c, 0x69, l, c);
  test_ld!(ld_l_d, 0x6A, l, d);
  test_ld!(ld_l_e, 0x6B, l, e);
  test_ld!(ld_l_h, 0x6C, l, h);
  test_ld!(ld_l_l, 0x6D, l, l);
  test_ld!(ld_l_a, 0x6F, l, a);

  test_ld!(ld_b_n, 0x06, b, n);
  test_ld!(ld_c_n, 0x0E, c, n);
  test_ld!(ld_d_n, 0x16, d, n);
  test_ld!(ld_e_n, 0x1E, e, n);
  test_ld!(ld_h_n, 0x26, h, n);
  test_ld!(ld_l_n, 0x2E, l, n);
  test_ld!(ld_a_n, 0x3E, a, n);

  test_ld!(ld_b_hl,0x46, b, (h l));
  test_ld!(ld_c_hl,0x4E, c, (h l));
  test_ld!(ld_d_hl,0x56, d, (h l));
  test_ld!(ld_e_hl,0x5E, e, (h l));
  test_ld!(ld_h_hl,0x66, h, (h l));
  test_ld!(ld_l_hl,0x6E, l, (h l));
  test_ld!(ld_a_hl,0x7E, a, (h l));

  test_ld!(ld_hl_b, 0x70, (h l), b);
  test_ld!(ld_hl_c, 0x71, (h l), c);
  test_ld!(ld_hl_d, 0x72, (h l), d);
  test_ld!(ld_hl_e, 0x73, (h l), e);
  test_ld!(ld_hl_h, 0x74, (h l), h);
  test_ld!(ld_hl_l, 0x75, (h l), l);
  test_ld!(ld_hl_a, 0x77, (h l), a);

  test_ld!(ld_hl_n,0x36, (hl), n);

  test_ld!(ld_a_bc, 0x0A, a, (b c));
  test_ld!(ld_a_de, 0x1A, a, (d e));
  test_ld!(ld_a_nn, 0xFA, a, (n n));

  test_ld!(ld_bc_a, 0x02, (b c), a);
  test_ld!(ld_de_a, 0x12, (d e), a);
  test_ld!(ld_nn_a, 0xEA, (n n), a);

  // XXX: these only test cycles for now.
  test_ld!(ld_a_ff00n, 0xF0, a, (0xFF00 + n));
  test_ld!(ld_ff00n_a, 0xE0, (0xFF00 + n), a);
  test_ld!(ld_a_ff00c, 0xF2, a, (0xFF00 + c));
  test_ld!(ld_ff00c_a, 0xE2, (0xFF00 + c), a);

  test_ldi!(ldi_hl_a, 0x22, (h l), a);
  test_ldi!(ldi_a_hl, 0x2A, a, (h l));
  test_ldd!(ldd_hl_a, 0x32, (h l), a);
  test_ldd!(ldd_a_hl, 0x3A, a, (h l));

  test_ld!(ld_bc_nn, 0x01, [b c], nn);
  test_ld!(ld_de_nn, 0x11, [d e], nn);
  test_ld!(ld_hl_nn, 0x21, [h l], nn);
  test_ld!(ld_sp_nn, 0x31, sp, nn);

  test_ld!(ld_sp_hl, 0xF9, sp, hl);

  test_push!(push_bc, 0xC5, [b c]);
  test_push!(push_de, 0xD5, [d e]);
  test_push!(push_hl, 0xE5, [h l]);
  test_push!(push_af, 0xF5, [a f]);

  test_pop!(pop_bc, 0xC1, [b c]);
  test_pop!(pop_de, 0xD1, [d e]);
  test_pop!(pop_hl, 0xE1, [h l]);
  test_pop!(pop_af, 0xF1, [a f]);

  test_add!(add_a_b, 0x80, b);
  test_add!(add_a_c, 0x81, c);
  test_add!(add_a_d, 0x82, d);
  test_add!(add_a_e, 0x83, e);
  test_add!(add_a_h, 0x84, h);
  test_add!(add_a_l, 0x85, l);
  test_add!(add_a_a, 0x87, a);

  test_add!(add_a_n, 0xC6, n);

  test_add!(add_a_hl, 0x86, hl);

  test_adc!(adc_a_b, 0x88, b);
  test_adc!(adc_a_c, 0x89, c);
  test_adc!(adc_a_d, 0x8A, d);
  test_adc!(adc_a_e, 0x8B, e);
  test_adc!(adc_a_h, 0x8C, h);
  test_adc!(adc_a_l, 0x8D, l);
  test_adc!(adc_a_a, 0x8F, a);

  test_adc!(adc_a_n, 0xCE, n);

  test_adc!(adc_a_hl, 0x8E, hl);

  test_sub!(sub_a_b, 0x90, b);
  test_sub!(sub_a_c, 0x91, c);
  test_sub!(sub_a_d, 0x92, d);
  test_sub!(sub_a_e, 0x93, e);
  test_sub!(sub_a_h, 0x94, h);
  test_sub!(sub_a_l, 0x95, l);
  test_sub!(sub_a_a, 0x97, a);

  test_sub!(sub_a_n, 0xD6, n);

  test_sub!(sub_a_hl, 0x96, hl);

  test_sbc!(sbc_a_b, 0x98, b);
  test_sbc!(sbc_a_c, 0x99, c);
  test_sbc!(sbc_a_d, 0x9A, d);
  test_sbc!(sbc_a_e, 0x9B, e);
  test_sbc!(sbc_a_h, 0x9C, h);
  test_sbc!(sbc_a_l, 0x9D, l);
  test_sbc!(sbc_a_a, 0x9F, a);

  test_sbc!(sbc_a_n, 0xDE, n);

  test_sbc!(sbc_a_hl, 0x9E, hl);

  test_and!(and_a_b, 0xA0, b);
  test_and!(and_a_c, 0xA1, c);
  test_and!(and_a_d, 0xA2, d);
  test_and!(and_a_e, 0xA3, e);
  test_and!(and_a_h, 0xA4, h);
  test_and!(and_a_l, 0xA5, l);
  test_and!(and_a_a, 0xA7, a);

  test_and!(and_a_n, 0xE6, n);

  test_and!(and_a_hl, 0xA6, hl);

  test_xor!(xor_a_b, 0xA8, b);
  test_xor!(xor_a_c, 0xA9, c);
  test_xor!(xor_a_d, 0xAA, d);
  test_xor!(xor_a_e, 0xAB, e);
  test_xor!(xor_a_h, 0xAC, h);
  test_xor!(xor_a_l, 0xAD, l);
  test_xor!(xor_a_a, 0xAF, a);

  test_xor!(xor_a_n, 0xEE, n);

  test_xor!(xor_a_hl, 0xAE, hl);

  test_or!(or_a_b, 0xB0, b);
  test_or!(or_a_c, 0xB1, c);
  test_or!(or_a_d, 0xB2, d);
  test_or!(or_a_e, 0xB3, e);
  test_or!(or_a_h, 0xB4, h);
  test_or!(or_a_l, 0xB5, l);
  test_or!(or_a_a, 0xB7, a);

  test_or!(or_a_n, 0xF6, n);

  test_or!(or_a_hl, 0xB6, hl);

  test_cp!(cp_a_b, 0xB8, b);
  test_cp!(cp_a_c, 0xB9, c);
  test_cp!(cp_a_d, 0xBA, d);
  test_cp!(cp_a_e, 0xBB, e);
  test_cp!(cp_a_h, 0xBC, h);
  test_cp!(cp_a_l, 0xBD, l);
  test_cp!(cp_a_a, 0xBF, a);

  test_cp!(cp_a_n, 0xFE, n);

  test_cp!(cp_a_hl, 0xBE, hl);

  test_inc!(inc_b, 0x04, b);
  test_inc!(inc_c, 0x0C, c);
  test_inc!(inc_d, 0x14, d);
  test_inc!(inc_e, 0x1C, e);
  test_inc!(inc_h, 0x24, h);
  test_inc!(inc_l, 0x2C, l);
  test_inc!(inc_hl_ind,0x34, hl);
  test_inc!(inc_a, 0x3C, a);

  test_dec!(dec_b, 0x05, b);
  test_dec!(dec_c, 0x0D, c);
  test_dec!(dec_d, 0x15, d);
  test_dec!(dec_e, 0x1D, e);
  test_dec!(dec_h, 0x25, h);
  test_dec!(dec_l, 0x2D, l);
  test_dec!(dec_hl_ind,0x35, hl);
  test_dec!(dec_a, 0x3D, a);

  test_add!(add_hl_bc, 0x09, [b c]);
  test_add!(add_hl_de, 0x19, [d e]);
  test_add!(add_hl_hl, 0x29, [h l]);
  test_add!(add_hl_sp, 0x39, sp);

  // test_add!(add_sp_dd, 0xE8, dd);

  test_inc!(inc_bc, 0x03, [b c]);
  test_inc!(inc_de, 0x13, [d e]);
  test_inc!(inc_hl, 0x23, [h l]);
  test_inc!(inc_sp, 0x33, sp);

  test_dec!(dec_bc, 0x0B, [b c]);
  test_dec!(dec_de, 0x1B, [d e]);
  test_dec!(dec_hl, 0x2B, [h l]);
  test_dec!(dec_sp, 0x3B, sp);
}
