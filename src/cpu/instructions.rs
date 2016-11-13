use cpu::registers::{R8, R16, FLAG};
use cpu::registers::R8::{A, B, C, D, E, F, H, L};
use cpu::registers::R16::{BC, DE, HL, SP, PC};
use cpu::registers::FLAG::{Z, N, H as HC, C as CY};
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
    let dd = self.read_pc() as i32;
    let sp = self.rr(SP) as i32;
    self.rr_set(HL, sp.wrapping_add(dd) as u16);
    self.f_clear(Z);
    self.f_clear(N);
    self.f_setb(HC, ((sp & 0x0F) + (dd & 0x0F)) > 0x0F);
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
    self.f_setb(HC, vl > 0x0F);
    self.f_setb(CY, vh > 0xFF);
  }

  pub fn op_add_a_r(&mut self, r: R8) -> u8 {
    let x = self.r(r);
    self._add(x);
    4
  }

  pub fn op_add_a_n(&mut self) -> u8 {
    let x = self.read_pc();
    self._add(x);
    8
  }

  pub fn op_add_a_hl(&mut self) -> u8 {
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
    self.f_setb(HC, vl > 0x0F);
    self.f_setb(CY, vh > 0xFF);
  }

  pub fn op_adc_a_r(&mut self, r: R8) -> u8 {
    let x = self.r(r);
    self._adc(x);
    4
  }

  pub fn op_adc_a_n(&mut self) -> u8 {
    let x = self.read_pc();
    self._adc(x);
    8
  }

  pub fn op_adc_a_hl(&mut self) -> u8 {
    let addr = self.rr(HL);
    let x = self.read(addr);
    self._adc(x);
    8
  }

  fn _sub(&mut self, x: u8) {
    let a = self.r(A);
    let vh = (a as u16).wrapping_sub(x as u16);
    let vl = (a & 0x0F).wrapping_sub(x & 0x0F);
    self.r_set(A, vh as u8);
    self.f_setb(Z, vh == 0);
    self.f_set(N);
    self.f_setb(HC, vl > 0x0F);
    self.f_setb(CY, vh > 0xFF);
  }

  pub fn op_sub_a_r(&mut self, r: R8) -> u8 {
    let x = self.r(r);
    self._sub(x);
    4
  }

  pub fn op_sub_a_n(&mut self) -> u8 {
    let x = self.read_pc();
    self._sub(x);
    8
  }

  pub fn op_sub_a_hl(&mut self) -> u8 {
    let addr = self.rr(HL);
    let x = self.read(addr);
    self._sub(x);
    8
  }

  // Rotate and shift

  fn _rlc(&mut self, x: u8) -> u8 {
    let v = x.rotate_left(1);
    self.f_clear(N);
    self.f_clear(HC);
    self.f_setb(CY, v & 0x01 > 0);
    v
  }

  fn _rl(&mut self, x: u8) -> u8 {
    let mut v = x.rotate_left(1);
    let c = v & 0x01 > 0;
    if self.f(CY) { v |= 1; }
    self.f_clear(N);
    self.f_clear(HC);
    self.f_setb(CY, c);
    v
  }

  fn _rrc(&mut self, x: u8) -> u8 {
    let v = x.rotate_right(1);
    self.f_clear(N);
    self.f_clear(HC);
    self.f_setb(CY, v & 0b1000_000 > 0);
    v
  }

  fn _rr(&mut self, x: u8) -> u8 {
    let mut v = x.rotate_right(1);
    let c = v & 0b1000_0000 > 0;
    if self.f(CY) { v |= 0b1000_0000; }
    self.f_clear(N);
    self.f_clear(HC);
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
    self.f_clear(HC);
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
    self.f_clear(HC);
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
    self.f_clear(HC);
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
    self.f_clear(HC);
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
    self.f_set(HC);
    8
  }

  pub fn op_bit_n_hl(&mut self, n: u8) -> u8 {
    let addr = self.rr(HL);
    let x = self.read(addr) & (1 << n);
    self.f_setb(Z, x == 0);
    self.f_clear(N);
    self.f_set(HC);
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
    self.f_clear(HC);
    self.f_setb(CY, !c);
    4
  }

  pub fn op_scf(&mut self) -> u8 {
    self.f_clear(N);
    self.f_clear(HC);
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
}



    macro_rules! inc {
      // INC (HL)
      ((h l)) => ({
        let addr = to_u16!(self.h, self.l);
        let v = self.read(addr);
        let r = v.wrapping_add(1);
        self.write(addr, r);
        z!(r);
        n!(0);
        h!(v, 1);
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
        z!(self.$r);
        n!(0);
        h!(v, 1);
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
        z!(r);
        n!(1);
        h!(v, 1, -);
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
        z!(self.$r);
        n!(1);
        h!(v, 1, -);
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
        let a = self.a as u16;
        let n = ($n) as u16;
        let rh = a + n;
        let rl = (a & 0xF) + (n & 0xF);
        self.a = rh as u8;
        z!(self.a);
        n!(0);
        h!(rl > 0x0F);
        c!(rh > 0xFF);
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

        // flags!(-0hc, r, rh);

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

        // flags!(-0hc, r, rh);

        let (h, l) = from_u16!(r as u16);
        self.h = h;
        self.l = l;
        cycles += 8;
      });

    }

    macro_rules! adc1 {
      ($n:expr) => ({
        let n = $n;
        let a = self.a;
        self.a = a.wrapping_add(n);
        if self.f & C_FLAG > 0 {
          self.a = self.a.wrapping_add(1);
        }
        z!(self.a);
        n!(0);
        h!(a, n, 1);
        c!(a, n, 1);
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
        let a = self.a;
        self.a = self.a.wrapping_sub(n);
        z!(self.a);
        n!(1);
        h!(a, n, -);
        c!(a, n, -);
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
        let a = self.a;

        self.a = self.a.wrapping_sub(n);
        if self.f & C_FLAG > 0 {
          self.a = self.a.wrapping_sub(1);
        }
        z!(self.a);
        n!(1);
        h!(a, n, -);
        c!(a, n, -);
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
        z!(self.a); n!(0); h!(1); c!(0);
        cycles += 8;
      });

      // AND A,n
      (n) => ({
        self.a &= self.read_pc();
        z!(self.a); n!(0); h!(1); c!(0);
        cycles += 8;
      });

      // AND A,r
      ($r:ident) => ({
        self.a &= self.$r;
        z!(self.a); n!(0); h!(1); c!(0);
        cycles += 4;
      });
    }

    macro_rules! xor {
      // XOR A,(HL)
      ((h l)) => ({
        let addr = to_u16!(self.h, self.l);
        self.a ^= self.read(addr);
        z!(self.a); n!(0); h!(0); c!(0);
        cycles += 8;
      });

      // XOR A,n
      (n) => ({
        self.a ^= self.read_pc();
        z!(self.a); n!(0); h!(0); c!(0);
        cycles += 8;
      });

      // XOR A,r
      ($r:ident) => ({
        self.a ^= self.$r;
        z!(self.a); n!(0); h!(0); c!(0);
        cycles += 4;
      });
    }

    macro_rules! or {
      // OR A,(HL)
      ((h l)) => ({
        let addr = to_u16!(self.h, self.l);
        self.a |= self.read(addr);
        z!(self.a); n!(0); h!(0); c!(0);
        cycles += 8;
      });

      // OR A,n
      (n) => ({
        self.a |= self.read_pc();
        z!(self.a); n!(0); h!(0); c!(0);
        cycles += 8;
      });

      // OR A,r
      ($r:ident) => ({
        self.a |= self.$r;
        z!(self.a); n!(0); h!(0); c!(0);
        cycles += 4;
      });
    }

    macro_rules! cp1 {
      ($n:expr) => ({
        let n = $n;
        let r = self.a.wrapping_sub(n);
        // flags!(z1hc, r, rh);
        z!(r);
        n!(1);
        h!(self.a, n, -);
        c!(self.a, n, -);
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

    macro_rules! cpl {
      () => ({
        unimplemented!();
      });
    }

    macro_rules! bit {
      ($n:expr, (h l)) => ({
        let addr = self.read_pc_16le();
        z!(self.read(addr) & (1 << ($n)));
        n!(0);
        h!(1);
      });

      ($n:expr, $r:ident) => ({
        z!(self.$r & (1 << ($n)));
        n!(0);
        h!(1);
      });
    }

    macro_rules! set {
      ($n:expr, (h l)) => ({
        let addr = self.read_pc_16le();
        let mut v = self.read(addr);
        v |= 1 << ($n);
        self.write(addr, v);
      });

      ($n:expr, $r:ident) => ({
        self.$r |= 1 << ($n);
      });
    }

    macro_rules! res {
      ($n:expr, (h l)) => ({
        let addr = self.read_pc_16le();
        let mut v = self.read(addr);
        v &= !(1 << ($n));
        self.write(addr, v);
      });

      ($n:expr, $r:ident) => ({
        self.$r &= !(1 << ($n));
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

    macro_rules! stop {
      () => ({
        unimplemented!();
      });
    }

    macro_rules! halt {
      () => ({
        unimplemented!();
      });
    }
