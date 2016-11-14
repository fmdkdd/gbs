use cpu::registers::{R8, R16, FLAG};
use cpu::registers::R8::{A, C};
use cpu::registers::R16::{HL, SP, PC};
use cpu::registers::FLAG::{Z, N, H as HY, C as CY};
use cpu::cpu::Cpu;

impl Cpu {

  // 8bit loads

  pub fn op_ld_r_r(&mut self, r1: R8, r2: R8) -> u8 {
    let v = self.r(r2);
    self.r_set(r1, v);
    4
  }

  pub fn op_ld_r_n(&mut self, r: R8) -> u8 {
    let v = self.read_pc();
    self.r_set(r, v);
    8
  }

  pub fn op_ld_r_rr(&mut self, r: R8, rr: R16) -> u8 {
    let addr = self.rr(rr);
    let v = self.read(addr);
    self.r_set(r, v);
    8
  }

  pub fn op_ld_hl_r(&mut self, r: R8) -> u8 {
    let v = self.r(r);
    let addr = self.rr(HL);
    self.write(addr, v);
    8
  }

  pub fn op_ld_hl_n(&mut self) -> u8 {
    let v = self.read_pc();
    let addr = self.rr(HL);
    self.write(addr, v);
    12
  }

  pub fn op_ld_a_nn(&mut self) -> u8 {
    let addr = self.read_pc_16le();
    let v = self.read(addr);
    self.r_set(A, v);
    16
  }

  pub fn op_ld_rr_a(&mut self, rr: R16) -> u8 {
    let addr = self.rr(rr);
    let v = self.r(A);
    self.write(addr, v);
    8
  }

  pub fn op_ld_nn_a(&mut self) -> u8 {
    let addr = self.read_pc_16le();
    let v = self.r(A);
    self.write(addr, v);
    16
  }

  pub fn op_ld_a_ffn(&mut self) -> u8 {
    let addr = 0xFF00 | (self.read_pc() as u16);
    let v = self.read(addr);
    self.r_set(A, v);
    12
  }

  pub fn op_ld_ffn_a(&mut self) -> u8 {
    let v = self.r(A);
    let addr = 0xFF00 | (self.read_pc() as u16);
    self.write(addr, v);
    12
  }

  pub fn op_ld_a_ffc(&mut self) -> u8 {
    let addr = 0xFF00 | (self.r(C) as u16);
    let v = self.read(addr);
    self.r_set(A, v);
    8
  }

  pub fn op_ld_ffc_a(&mut self) -> u8 {
    let v = self.r(A);
    let addr = 0xFF00 | (self.r(C) as u16);
    self.write(addr, v);
    8
  }

  pub fn op_ldi_hl_a(&mut self) -> u8 {
    let addr = self.rr(HL);
    let v = self.r(A);
    self.write(addr, v);
    self.rr_set(HL, addr.wrapping_add(1));
    8
  }

  pub fn op_ldi_a_hl(&mut self) -> u8 {
    let addr = self.rr(HL);
    let v = self.read(addr);
    self.r_set(A, v);
    self.rr_set(HL, addr.wrapping_add(1));
    8
  }

  pub fn op_ldd_hl_a(&mut self) -> u8 {
    let addr = self.rr(HL);
    let v = self.r(A);
    self.write(addr, v);
    self.rr_set(HL, addr.wrapping_sub(1));
    8
  }

  pub fn op_ldd_a_hl(&mut self) -> u8 {
    let addr = self.rr(HL);
    let v = self.read(addr);
    self.r_set(A, v);
    self.rr_set(HL, addr.wrapping_sub(1));
    8
  }

  // 16bit loads

  pub fn op_ld_rr_nn(&mut self, rr: R16) -> u8 {
    let v = self.read_pc_16le();
    self.rr_set(rr, v);
    12
  }

  pub fn op_ld_sp_hl(&mut self) -> u8 {
    let v = self.rr(HL);
    self.rr_set(SP, v);
    8
  }

  pub fn op_ld_hl_sp_dd(&mut self) -> u8 {
    // Casting to i32 directly loses the sign
    let dd = (self.read_pc() as i8) as i32;
    // Casting to i16 would add an unwanted sign
    let sp = self.rr(SP) as i32;
    self.rr_set(HL, sp.wrapping_add(dd) as u16);
    self.f_clear(Z);
    self.f_clear(N);
    self.f_setb(HY, ((sp & 0x0F) + (dd & 0x0F)) > 0x0F);
    self.f_setb(CY, ((sp & 0xFF) + (dd & 0xFF)) > 0xFF);
    12
  }

  pub fn op_ld_nn_sp(&mut self) -> u8 {
    let v = self.rr(SP);
    let addr = self.read_pc_16le();
    self.write_16le(addr, v);
    20
  }

  pub fn op_push_rr(&mut self, rr: R16) -> u8 {
    let sp = self.rr(SP).wrapping_sub(2);
    self.rr_set(SP, sp);
    let v = self.rr(rr);
    self.write_16le(sp, v);
    16
  }

  pub fn op_pop_rr(&mut self, rr: R16) -> u8 {
    let sp = self.rr(SP);
    let v = self.read_16le(sp);
    self.rr_set(rr, v);
    self.rr_set(SP, sp.wrapping_add(2));
    // Flags are set automatically when writing whatever to AF
    12
  }

  // 8bit arithmetic

  fn _add(&mut self, x: u8) {
    let a = self.r(A);
    let vh = (a as u16) + (x as u16);
    let vl = (a & 0x0F) + (x & 0x0F);
    self.r_set(A, vh as u8);
    self.f_setb(Z, vh == 0);
    self.f_clear(N);
    self.f_setb(HY, vl > 0x0F);
    self.f_setb(CY, vh > 0xFF);
  }

  pub fn op_add_r(&mut self, r: R8) -> u8 {
    let x = self.r(r);
    self._add(x);
    4
  }

  pub fn op_add_n(&mut self) -> u8 {
    let x = self.read_pc();
    self._add(x);
    8
  }

  pub fn op_add_hl(&mut self) -> u8 {
    let addr = self.rr(HL);
    let x = self.read(addr);
    self._add(x);
    8
  }

  fn _adc(&mut self, x: u8) {
    let a = self.r(A);
    let c : u8 = if self.f(CY) { 1 } else { 0 };
    let vh = (a as u16) + (x as u16) + (c as u16);
    let vl = (a & 0x0F) + (x & 0x0F) + c;
    self.r_set(A, vh as u8);
    self.f_setb(Z, vh == 0);
    self.f_clear(N);
    self.f_setb(HY, vl > 0x0F);
    self.f_setb(CY, vh > 0xFF);
  }

  pub fn op_adc_r(&mut self, r: R8) -> u8 {
    let x = self.r(r);
    self._adc(x);
    4
  }

  pub fn op_adc_n(&mut self) -> u8 {
    let x = self.read_pc();
    self._adc(x);
    8
  }

  pub fn op_adc_hl(&mut self) -> u8 {
    let addr = self.rr(HL);
    let x = self.read(addr);
    self._adc(x);
    8
  }

  fn _sub(&mut self, x: u8) -> u8 {
    let a = self.r(A);
    let vh = (a as u16).wrapping_sub(x as u16);
    let vl = (a & 0x0F).wrapping_sub(x & 0x0F);
    self.f_setb(Z, vh == 0);
    self.f_set(N);
    self.f_setb(HY, vl > 0x0F);
    self.f_setb(CY, vh > 0xFF);
    vh as u8
  }

  pub fn op_sub_r(&mut self, r: R8) -> u8 {
    let x = self.r(r);
    let v = self._sub(x);
    self.r_set(A, v);
    4
  }

  pub fn op_sub_n(&mut self) -> u8 {
    let x = self.read_pc();
    let v = self._sub(x);
    self.r_set(A, v);
    8
  }

  pub fn op_sub_hl(&mut self) -> u8 {
    let addr = self.rr(HL);
    let x = self.read(addr);
    let v = self._sub(x);
    self.r_set(A, v);
    8
  }

  fn _sbc(&mut self, x: u8) {
    let a = self.r(A);
    let c : u8 = if self.f(CY) { 1 } else { 0 };
    let vh = (a as u16).wrapping_sub(x as u16).wrapping_sub(c as u16);
    let vl = (a & 0x0F).wrapping_sub(x & 0x0F).wrapping_sub(c);
    self.r_set(A, vh as u8);
    self.f_setb(Z, vh == 0);
    self.f_set(N);
    self.f_setb(HY, vl > 0x0F);
    self.f_setb(CY, vh > 0xFF);
  }

  pub fn op_sbc_r(&mut self, r: R8) -> u8 {
    let x = self.r(r);
    self._sbc(x);
    4
  }

  pub fn op_sbc_n(&mut self) -> u8 {
    let x = self.read_pc();
    self._sbc(x);
    8
  }

  pub fn op_sbc_hl(&mut self) -> u8 {
    let addr = self.rr(HL);
    let x = self.read(addr);
    self._sbc(x);
    8
  }

  fn _and(&mut self, x: u8) {
    let v = self.r(A) & x;
    self.r_set(A, v);
    self.f_setb(Z, v == 0);
    self.f_clear(N);
    self.f_set(HY);
    self.f_clear(CY);
  }

  pub fn op_and_r(&mut self, r: R8) -> u8 {
    let x = self.r(r);
    self._and(x);
    4
  }

  pub fn op_and_n(&mut self) -> u8 {
    let x = self.read_pc();
    self._and(x);
    8
  }

  pub fn op_and_hl(&mut self) -> u8 {
    let addr = self.rr(HL);
    let x = self.read(addr);
    self._and(x);
    8
  }

  fn _xor(&mut self, x: u8) {
    let v = self.r(A) ^ x;
    self.r_set(A, v);
    self.f_setb(Z, v == 0);
    self.f_clear(N);
    self.f_clear(HY);
    self.f_clear(CY);
  }

  pub fn op_xor_r(&mut self, r: R8) -> u8 {
    let x = self.r(r);
    self._xor(x);
    4
  }

  pub fn op_xor_n(&mut self) -> u8 {
    let x = self.read_pc();
    self._xor(x);
    8
  }

  pub fn op_xor_hl(&mut self) -> u8 {
    let addr = self.rr(HL);
    let x = self.read(addr);
    self._xor(x);
    8
  }

  fn _or(&mut self, x: u8) {
    let v = self.r(A) | x;
    self.r_set(A, v);
    self.f_setb(Z, v == 0);
    self.f_clear(N);
    self.f_clear(HY);
    self.f_clear(CY);
  }

  pub fn op_or_r(&mut self, r: R8) -> u8 {
    let x = self.r(r);
    self._xor(x);
    4
  }

  pub fn op_or_n(&mut self) -> u8 {
    let x = self.read_pc();
    self._xor(x);
    8
  }

  pub fn op_or_hl(&mut self) -> u8 {
    let addr = self.rr(HL);
    let x = self.read(addr);
    self._xor(x);
    8
  }

  pub fn op_cp_r(&mut self, r: R8) -> u8 {
    let x = self.r(r);
    self._sub(x);
    4
  }

  pub fn op_cp_n(&mut self) -> u8 {
    let x = self.read_pc();
    self._sub(x);
    4
  }

  pub fn op_cp_hl(&mut self) -> u8 {
    let addr = self.rr(HL);
    let x = self.read(addr);
    self._sub(x);
    8
  }

  fn _inc(&mut self, x: u8) -> u8 {
    let v = x.wrapping_add(1);
    self.f_setb(Z, v == 0);
    self.f_clear(N);
    self.f_setb(HY, (v & 0x0F) == 0);
    v
  }

  pub fn op_inc_r(&mut self, r: R8) -> u8 {
    let x = self.r(r);
    let v = self._inc(x);
    self.r_set(r, v);
    4
  }

  pub fn op_inc_hl(&mut self) -> u8 {
    let addr = self.rr(HL);
    let x = self.read(addr);
    let v = self._inc(x);
    self.write(addr, v);
    12
  }

  fn _dec(&mut self, x: u8) -> u8 {
    let v = x.wrapping_sub(1);
    self.f_setb(Z, v == 0);
    self.f_set(N);
    self.f_setb(HY, (v & 0x0F) == 0x0F);
    v
  }

  pub fn op_dec_r(&mut self, r: R8) -> u8 {
    let x = self.r(r);
    let v = self._dec(x);
    self.r_set(r, v);
    4
  }

  pub fn op_dec_hl(&mut self) -> u8 {
    let addr = self.rr(HL);
    let x = self.read(addr);
    let v = self._dec(x);
    self.write(addr, v);
    12
  }

  pub fn op_daa(&mut self) -> u8 {
    // Code lifted from Higan, because specs are glossing over this one
    let mut a = self.r(A) as u16;
    if self.f(N) {
      if self.f(HY) || ((a & 0x0F) > 0x09) { a = a.wrapping_add(0x06) }
      if self.f(CY) || ((a       ) > 0x9F) { a = a.wrapping_add(0x60) }
    } else {
      if self.f(HY) {
        a = a.wrapping_sub(0x06);
        if !self.f(CY) { a &= 0xFF }
      }
      if self.f(CY) { a = a.wrapping_sub(0xFF) }
    }
    self.r_set(A, a as u8);
    self.f_setb(Z, a == 0);
    self.f_clear(HY);
    self.f_setb(CY, a & 0x100 > 0);
    4
  }

  pub fn op_cpl(&mut self) -> u8 {
    let v = self.r(A) ^ 0xFF;
    self.r_set(A, v);
    self.f_set(N);
    self.f_set(HY);
    4
  }

  // 16bit arithmetic

  pub fn op_add_hl_rr(&mut self, rr: R16) -> u8 {
    let hl = self.rr(HL) as u32;
    let x = self.rr(rr) as u32;
    let vh = hl + x;
    let vl = (hl & 0x0FFF) + (x & 0x0FFF);
    self.rr_set(HL, vh as u16);
    self.f_clear(N);
    self.f_setb(HY, vl > 0x0FFF);
    self.f_setb(CY, vh > 0xFFFF);
    8
  }

  pub fn op_inc_rr(&mut self, rr: R16) -> u8 {
    let x = self.rr(rr);
    self.rr_set(rr, x.wrapping_add(1));
    8
  }

  pub fn op_dec_rr(&mut self, rr: R16) -> u8 {
    let x = self.rr(rr);
    self.rr_set(rr, x.wrapping_sub(1));
    8
  }

  pub fn op_add_sp_dd(&mut self) -> u8 {
    // Casting to i32 directly loses the sign
    let dd = (self.read_pc() as i8) as i32;
    // Casting to i16 would add an unwanted sign
    let sp = self.rr(SP) as i32;
    self.rr_set(SP, sp.wrapping_add(dd) as u16);
    self.f_clear(Z);
    self.f_clear(N);
    self.f_setb(HY, ((sp & 0x0F) + (dd & 0x0F)) > 0x0F);
    self.f_setb(CY, ((sp & 0xFF) + (dd & 0xFF)) > 0xFF);
    16
  }

  // Rotate and shift

  fn _rlc(&mut self, x: u8) -> u8 {
    let v = x.rotate_left(1);
    self.f_clear(N);
    self.f_clear(HY);
    self.f_setb(CY, v & 0x01 > 0);
    v
  }

  fn _rl(&mut self, x: u8) -> u8 {
    let mut v = x.rotate_left(1);
    let c = v & 0x01 > 0;
    if self.f(CY) { v |= 1; }
    self.f_clear(N);
    self.f_clear(HY);
    self.f_setb(CY, c);
    v
  }

  fn _rrc(&mut self, x: u8) -> u8 {
    let v = x.rotate_right(1);
    self.f_clear(N);
    self.f_clear(HY);
    self.f_setb(CY, v & 0b1000_000 > 0);
    v
  }

  fn _rr(&mut self, x: u8) -> u8 {
    let mut v = x.rotate_right(1);
    let c = v & 0b1000_0000 > 0;
    if self.f(CY) { v |= 0b1000_0000; }
    self.f_clear(N);
    self.f_clear(HY);
    self.f_setb(CY, c);
    v
  }

  pub fn op_rlca(&mut self) -> u8 {
    let a = self.r(A);
    let v = self._rlc(a);
    self.r_set(A, v);
    self.f_clear(Z);
    4
  }

  pub fn op_rla(&mut self) -> u8 {
    let a = self.r(A);
    let v = self._rl(a);
    self.r_set(A, v);
    self.f_clear(Z);
    4
  }

  pub fn op_rrca(&mut self) -> u8 {
    let a = self.r(A);
    let v = self._rrc(a);
    self.r_set(A, v);
    self.f_clear(Z);
    4
  }

  pub fn op_rra(&mut self) -> u8 {
    let a = self.r(A);
    let v = self._rr(a);
    self.r_set(A, v);
    self.f_clear(Z);
    4
  }

  pub fn op_rlc_r(&mut self, r: R8) -> u8 {
    let x = self.r(r);
    let v = self._rlc(x);
    self.r_set(r, v);
    self.f_setb(Z, v == 0);
    8
  }

  pub fn op_rlc_hl(&mut self) -> u8 {
    let addr = self.rr(HL);
    let x = self.read(addr);
    let v = self._rlc(x);
    self.write(addr, v);
    self.f_setb(Z, v == 0);
    16
  }

  pub fn op_rl_r(&mut self, r: R8) -> u8 {
    let x = self.r(r);
    let v = self._rl(x);
    self.r_set(r, v);
    self.f_setb(Z, v == 0);
    8
  }

  pub fn op_rl_hl(&mut self) -> u8 {
    let addr = self.rr(HL);
    let x = self.read(addr);
    let v = self._rl(x);
    self.write(addr, v);
    self.f_setb(Z, v == 0);
    16
  }

  pub fn op_rrc_r(&mut self, r: R8) -> u8 {
    let x = self.r(r);
    let v = self._rrc(x);
    self.r_set(r, v);
    self.f_setb(Z, v == 0);
    8
  }

  pub fn op_rrc_hl(&mut self) -> u8 {
    let addr = self.rr(HL);
    let x = self.read(addr);
    let v = self._rrc(x);
    self.write(addr, v);
    self.f_setb(Z, v == 0);
    16
  }

  pub fn op_rr_r(&mut self, r: R8) -> u8 {
    let x = self.r(r);
    let v = self._rr(x);
    self.r_set(r, v);
    self.f_setb(Z, v == 0);
    8
  }

  pub fn op_rr_hl(&mut self) -> u8 {
    let addr = self.rr(HL);
    let x = self.read(addr);
    let v = self._rr(x);
    self.write(addr, v);
    self.f_setb(Z, v == 0);
    16
  }

  fn _sla(&mut self, x: u8) -> u8 {
    self.f_setb(CY, x & 0b1000_0000 > 0);
    let v = x << 1;
    self.f_setb(Z, v == 0);
    self.f_clear(N);
    self.f_clear(HY);
    v
  }

  pub fn op_sla_r(&mut self, r: R8) -> u8 {
    let x = self.r(r);
    let v = self._sla(x);
    self.r_set(r, v);
    8
  }

  pub fn op_sla_hl(&mut self) -> u8 {
    let addr = self.rr(HL);
    let x = self.read(addr);
    let v = self._sla(x);
    self.write(addr, v);
    16
  }

  fn _swap(&mut self, x: u8) -> u8 {
    let v = x.rotate_right(4);
    self.f_setb(Z, v == 0);
    self.f_clear(N);
    self.f_clear(HY);
    self.f_clear(CY);
    v
  }

  pub fn op_swap_r(&mut self, r: R8) -> u8 {
    let x = self.r(r);
    let v = self._swap(x);
    self.r_set(r, v);
    8
  }

  pub fn op_swap_hl(&mut self) -> u8 {
    let addr = self.rr(HL);
    let x = self.read(addr);
    let v = self._swap(x);
    self.write(addr, v);
    16
  }

  fn _sra(&mut self, x: u8) -> u8 {
    self.f_setb(CY, x & 0x01 > 0);
    let v = (x as i8) >> 1;
    self.f_setb(Z, v == 0);
    self.f_clear(N);
    self.f_clear(HY);
    v as u8
  }

  pub fn op_sra_r(&mut self, r: R8) -> u8 {
    let x = self.r(r);
    let v = self._sra(x);
    self.r_set(r, v);
    8
  }

  pub fn op_sra_hl(&mut self) -> u8 {
    let addr = self.rr(HL);
    let x = self.read(addr);
    let v = self._sra(x);
    self.write(addr, v);
    16
  }

  fn _srl(&mut self, x: u8) -> u8 {
    self.f_setb(CY, x & 0x01 > 0);
    let v = x >> 1;
    self.f_setb(Z, v == 0);
    self.f_clear(N);
    self.f_clear(HY);
    v
  }

  pub fn op_srl_r(&mut self, r: R8) -> u8 {
    let x = self.r(r);
    let v = self._srl(x);
    self.r_set(r, v);
    8
  }

  pub fn op_srl_hl(&mut self) -> u8 {
    let addr = self.rr(HL);
    let x = self.read(addr);
    let v = self._srl(x);
    self.write(addr, v);
    16
  }

  // Bit ops

  pub fn op_bit_n_r(&mut self, n: u8, r: R8) -> u8 {
    let x = self.r(r) & (1 << n);
    self.f_setb(Z, x == 0);
    self.f_clear(N);
    self.f_set(HY);
    8
  }

  pub fn op_bit_n_hl(&mut self, n: u8) -> u8 {
    let addr = self.rr(HL);
    let x = self.read(addr) & (1 << n);
    self.f_setb(Z, x == 0);
    self.f_clear(N);
    self.f_set(HY);
    12
  }

  pub fn op_set_n_r(&mut self, n: u8, r: R8) -> u8 {
    let x = self.r(r) | (1 << n);
    self.r_set(r, x);
    8
  }

  pub fn op_set_n_hl(&mut self, n: u8) -> u8 {
    let addr = self.rr(HL);
    let x = self.read(addr) | (1 << n);
    self.write(addr, x);
    16
  }

  pub fn op_res_n_r(&mut self, n: u8, r: R8) -> u8 {
    let x = self.r(r) & !(1 << n);
    self.r_set(r, x);
    8
  }

  pub fn op_res_n_hl(&mut self, n: u8) -> u8 {
    let addr = self.rr(HL);
    let x = self.read(addr) & !(1 << n);
    self.write(addr, x);
    16
  }

  // Control

  pub fn op_ccf(&mut self) -> u8 {
    let c = self.f(CY);
    self.f_clear(N);
    self.f_clear(HY);
    self.f_setb(CY, !c);
    4
  }

  pub fn op_scf(&mut self) -> u8 {
    self.f_clear(N);
    self.f_clear(HY);
    self.f_set(CY);
    4
  }

  pub fn op_nop(&self) -> u8 {
    4
  }

  pub fn op_stop(&self) -> u8 {
    // unimplemented!();
    4
  }

  pub fn op_halt(&self) -> u8 {
    // unimplemented!();
    4
  }

  pub fn op_di(&mut self) -> u8 {
    self.ime = 0;
    4
  }

  pub fn op_ei(&mut self) -> u8 {
    self.ime = 1;
    4
  }

  // Jumps

  pub fn op_jp_nn(&mut self) -> u8 {
    let nn = self.read_pc_16le();
    self.rr_set(PC, nn);
    16
  }

  pub fn op_jp_hl(&mut self) -> u8 {
    let hl = self.rr(HL);
    self.rr_set(PC, hl);
    4
  }

  pub fn op_jp_f_nn(&mut self, f: FLAG, b: bool) -> u8 {
    let nn = self.read_pc_16le();
    let mut cycles = 12;
    if self.f(f) == b {
      self.rr_set(PC, nn);
      cycles += 4;
    }
    cycles
  }

  pub fn op_jr_dd(&mut self) -> u8 {
    // Casting to i32 directly loses the sign
    let dd = (self.read_pc() as i8) as i32;
    // Casting to i16 would add an unwanted sign
    let pc = self.rr(PC) as i32;
    self.rr_set(PC, pc.wrapping_add(dd) as u16);
    12
  }

  pub fn op_jr_f_dd(&mut self, f: FLAG, b: bool) -> u8 {
    // Casting to i32 directly loses the sign
    let dd = (self.read_pc() as i8) as i32;
    // Casting to i16 would add an unwanted sign
    let pc = self.rr(PC) as i32;
    let mut cycles = 8;
    if self.f(f) == b {
      self.rr_set(PC, pc.wrapping_add(dd) as u16);
      cycles += 4;
    }
    cycles
  }

  pub fn op_call_nn(&mut self) -> u8 {
    let nn = self.read_pc_16le();
    let sp = self.rr(SP).wrapping_sub(2);
    let pc = self.rr(PC);
    self.rr_set(SP, sp);
    self.write_16le(sp, pc);
    self.rr_set(PC, nn);
    24
  }

  pub fn op_call_f_nn(&mut self, f: FLAG, b: bool) -> u8 {
    let nn = self.read_pc_16le();
    let mut cycles = 12;
    if self.f(f) == b {
      let sp = self.rr(SP).wrapping_sub(2);
      let pc = self.rr(PC);
      self.rr_set(SP, sp);
      self.write_16le(sp, pc);
      self.rr_set(PC, nn);
      cycles += 12;
    }
    cycles
  }

  pub fn op_ret(&mut self) -> u8 {
    let sp = self.rr(SP);
    let pc = self.read_16le(sp);
    self.rr_set(SP, sp.wrapping_add(2));
    self.rr_set(PC, pc);
    16
  }

  pub fn op_ret_f(&mut self, f: FLAG, b: bool) -> u8 {
    let mut cycles = 8;
    if self.f(f) == b {
      let sp = self.rr(SP);
      let pc = self.read_16le(sp);
      self.rr_set(SP, sp.wrapping_add(2));
      self.rr_set(PC, pc);
      cycles += 12;
    }
    cycles
  }

  pub fn op_reti(&mut self) -> u8 {
    self.ime = 1;
    self.op_ret()
  }

  pub fn op_rst(&mut self, n: u8) -> u8 {
    let sp = self.rr(SP).wrapping_sub(2);
    let pc = self.rr(PC);
    self.write_16le(sp, pc);
    self.rr_set(PC, n as u16);
    16
  }

}
