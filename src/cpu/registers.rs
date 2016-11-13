use utils::{to_u16, from_u16};

const Z_FLAG : u8 = 0b1000_0000;
const N_FLAG : u8 = 0b0100_0000;
const H_FLAG : u8 = 0b0010_0000;
const C_FLAG : u8 = 0b0001_0000;

#[derive(Copy, Clone)]
pub enum R8 {
  A, B, C, D, E, F, H, L
}

#[derive(Copy, Clone)]
pub enum R16 {
  AF, BC, DE, HL, SP, PC,
}

#[derive(Copy, Clone)]
pub enum FLAG {
  Z, N, H, C,
}

pub struct Registers {
  a: u8,
  b: u8,
  c: u8,
  d: u8,
  e: u8,
  f: u8,
  h: u8,
  l: u8,
  sp: u16,
  pc: u16,
}

impl Registers {
  pub fn new() -> Self {
    Registers {
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
    }
  }

  pub fn r(&self, r: R8) -> u8 {
    match r {
      R8::A => self.a,
      R8::B => self.b,
      R8::C => self.c,
      R8::D => self.d,
      R8::E => self.e,
      R8::F => self.f,
      R8::H => self.h,
      R8::L => self.l,
    }
  }

  pub fn r_set(&mut self, r: R8, x: u8) {
    match r {
      R8::A => self.a = x,
      R8::B => self.b = x,
      R8::C => self.c = x,
      R8::D => self.d = x,
      R8::E => self.e = x,
      R8::F => self.f = x,
      R8::H => self.h = x,
      R8::L => self.l = x,
    }
  }

  pub fn rr(&self, rr: R16) -> u16 {
    match rr {
      R16::AF => to_u16(self.a, self.f),
      R16::BC => to_u16(self.b, self.c),
      R16::DE => to_u16(self.d, self.e),
      R16::HL => to_u16(self.h, self.l),
      R16::SP => self.sp,
      R16::PC => self.pc,
    }
  }

  pub fn rr_set(&mut self, rr: R16, x: u16) {
    match rr {
      R16::AF => { let (h, l) = from_u16(x); self.a = h; self.f = l; },
      R16::BC => { let (h, l) = from_u16(x); self.b = h; self.c = l; },
      R16::DE => { let (h, l) = from_u16(x); self.d = h; self.e = l; },
      R16::HL => { let (h, l) = from_u16(x); self.h = h; self.l = l; },
      R16::SP => self.sp = x,
      R16::PC => self.pc = x,
    }
  }

  pub fn f(&self, f: FLAG) -> bool {
    match f {
      FLAG::Z => self.f & Z_FLAG > 0,
      FLAG::N => self.f & N_FLAG > 0,
      FLAG::H => self.f & H_FLAG > 0,
      FLAG::C => self.f & C_FLAG > 0,
    }
  }

  pub fn f_set(&mut self, f: FLAG) {
    match f {
      FLAG::Z => self.f |= Z_FLAG,
      FLAG::N => self.f |= N_FLAG,
      FLAG::H => self.f |= H_FLAG,
      FLAG::C => self.f |= C_FLAG,
    }
  }

  pub fn f_clear(&mut self, f: FLAG) {
    match f {
      FLAG::Z => self.f &= !Z_FLAG,
      FLAG::N => self.f &= !N_FLAG,
      FLAG::H => self.f &= !H_FLAG,
      FLAG::C => self.f &= !C_FLAG,
    }
  }

  pub fn f_setb(&mut self, f: FLAG, b: bool) {
    if b { self.f_set(f); } else { self.f_clear(f); }
  }
}
