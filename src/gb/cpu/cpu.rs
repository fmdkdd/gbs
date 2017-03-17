use gb::cpu::registers::{Registers, R8, R16, FLAG};
use gb::cpu::registers::R8::*;
use gb::cpu::registers::R16::*;
use gb::bus::Bus;
use gb::utils::{from_u16, to_u16};

pub struct Cpu<B: Bus> {
  r: Registers,
  pub ime: u8,
  pub bus: B,
}

impl<B> Cpu<B> where B: Bus {
  pub fn new(bus: B) -> Cpu<B> {
    Cpu {
      r: Registers::new(),
      ime: 0,
      bus: bus,
    }
  }

  // Expose these to avoid making self.r public
  pub fn r(&self, r: R8) -> u8 { self.r.r(r) }
  pub fn rr(&self, rr: R16) -> u16 { self.r.rr(rr) }
  pub fn f(&self, f: FLAG) -> bool { self.r.f(f) }
  pub fn r_set(&mut self, r: R8, x: u8) { self.r.r_set(r, x); }
  pub fn rr_set(&mut self, rr: R16, x: u16) { self.r.rr_set(rr, x); }
  pub fn f_set(&mut self, f: FLAG) { self.r.f_set(f); }
  pub fn f_clear(&mut self, f: FLAG) { self.r.f_clear(f); }
  pub fn f_setb(&mut self, f: FLAG, b: bool) { self.r.f_setb(f, b); }

  pub fn reset(&mut self) {
    self.rr_set(AF, 0);
    self.rr_set(BC, 0);
    self.rr_set(DE, 0);
    self.rr_set(HL, 0);
    self.rr_set(PC, 0);
    self.rr_set(SP, 0);
  }

  pub fn read_pc(&mut self) -> u8 {
    let pc = self.rr(PC);
    let ret = self.read(pc);
    self.rr_set(PC, pc.wrapping_add(1));
    ret
  }

  pub fn read_pc_16le(&mut self) -> u16 {
    let l = self.read_pc();
    let h = self.read_pc();
    to_u16(h, l)
  }

  pub fn read(&self, addr: u16) -> u8 {
    self.bus.read(addr)
  }

  pub fn write(&mut self, addr: u16, w: u8) {
    self.bus.write(addr, w);
  }

  pub fn read_16le(&self, addr: u16) -> u16 {
    let l = self.read(addr);
    let h = self.read((addr.wrapping_add(1)));
    to_u16(h, l)
  }

  pub fn write_16le(&mut self, addr: u16, ww: u16) {
    let (h, l) = from_u16(ww);
    self.write(addr, l);
    self.write(addr.wrapping_add(1), h);
  }

  // Run the next instruction and return the number of CPU cycles it took
  pub fn step(&mut self) -> u8 {
    let opcode = self.read_pc();

    if cfg!(feature = "debug") {
      println!("{:04x} {}\tAF:{:04x} BC:{:04x} DE:{:04x} HL:{:04x} SP:{:04x}",
               self.rr(PC) - 1, self.decode(opcode), self.rr(AF),
               self.rr(BC), self.rr(DE), self.rr(HL), self.rr(SP));
    }

    match opcode {
      // Following the table at
      // http://www.pastraiser.com/cpu/gameboy/gameboy_opcodes.html
      // Since none of the docs I found will actually list the exact opcodes,
      // just the mnemonics.

      0x00 => self.op_nop(),
      0x10 => self.op_stop(),
      0x20 => self.op_jr_f_dd(FLAG::Z, false),
      0x30 => self.op_jr_f_dd(FLAG::C, false),

      0x01 => self.op_ld_rr_nn(BC),
      0x11 => self.op_ld_rr_nn(DE),
      0x21 => self.op_ld_rr_nn(HL),
      0x31 => self.op_ld_rr_nn(SP),

      0x02 => self.op_ld_rr_a(BC),
      0x12 => self.op_ld_rr_a(DE),
      0x22 => self.op_ldi_hl_a(),
      0x32 => self.op_ldd_hl_a(),

      0x03 => self.op_inc_rr(BC),
      0x13 => self.op_inc_rr(DE),
      0x23 => self.op_inc_rr(HL),
      0x33 => self.op_inc_rr(SP),

      0x04 => self.op_inc_r(B),
      0x14 => self.op_inc_r(D),
      0x24 => self.op_inc_r(H),
      0x34 => self.op_inc_hl(),

      0x05 => self.op_dec_r(B),
      0x15 => self.op_dec_r(D),
      0x25 => self.op_dec_r(H),
      0x35 => self.op_dec_hl(),

      0x06 => self.op_ld_r_n(B),
      0x16 => self.op_ld_r_n(D),
      0x26 => self.op_ld_r_n(H),
      0x36 => self.op_ld_hl_n(),

      0x07 => self.op_rlca(),
      0x17 => self.op_rla(),
      0x27 => self.op_daa(),
      0x37 => self.op_scf(),

      0x08 => self.op_ld_nn_sp(),
      0x18 => self.op_jr_dd(),
      0x28 => self.op_jr_f_dd(FLAG::Z, true),
      0x38 => self.op_jr_f_dd(FLAG::Z, true),

      0x09 => self.op_add_hl_rr(BC),
      0x19 => self.op_add_hl_rr(DE),
      0x29 => self.op_add_hl_rr(HL),
      0x39 => self.op_add_hl_rr(SP),

      0x0A => self.op_ld_r_rr(A, BC),
      0x1A => self.op_ld_r_rr(A, DE),
      0x2A => self.op_ldi_a_hl(),
      0x3A => self.op_ldd_a_hl(),

      0x0B => self.op_dec_rr(BC),
      0x1B => self.op_dec_rr(DE),
      0x2B => self.op_dec_rr(HL),
      0x3B => self.op_dec_rr(SP),

      0x0C => self.op_inc_r(C),
      0x1C => self.op_inc_r(E),
      0x2C => self.op_inc_r(L),
      0x3C => self.op_inc_r(A),

      0x0D => self.op_dec_r(C),
      0x1D => self.op_dec_r(E),
      0x2D => self.op_dec_r(L),
      0x3D => self.op_dec_r(A),

      0x0E => self.op_ld_r_n(C),
      0x1E => self.op_ld_r_n(E),
      0x2E => self.op_ld_r_n(L),
      0x3E => self.op_ld_r_n(A),

      0x0F => self.op_rrca(),
      0x1F => self.op_rra(),
      0x2F => self.op_cpl(),
      0x3F => self.op_ccf(),

      0x40 => self.op_ld_r_r(B, B),
      0x41 => self.op_ld_r_r(B, C),
      0x42 => self.op_ld_r_r(B, D),
      0x43 => self.op_ld_r_r(B, E),
      0x44 => self.op_ld_r_r(B, H),
      0x45 => self.op_ld_r_r(B, L),
      0x46 => self.op_ld_r_rr(B, HL),
      0x47 => self.op_ld_r_r(B, A),

      0x48 => self.op_ld_r_r(C, B),
      0x49 => self.op_ld_r_r(C, C),
      0x4A => self.op_ld_r_r(C, D),
      0x4B => self.op_ld_r_r(C, E),
      0x4C => self.op_ld_r_r(C, H),
      0x4D => self.op_ld_r_r(C, L),
      0x4E => self.op_ld_r_rr(C, HL),
      0x4F => self.op_ld_r_r(C, A),

      0x50 => self.op_ld_r_r(D, B),
      0x51 => self.op_ld_r_r(D, C),
      0x52 => self.op_ld_r_r(D, D),
      0x53 => self.op_ld_r_r(D, E),
      0x54 => self.op_ld_r_r(D, H),
      0x55 => self.op_ld_r_r(D, L),
      0x56 => self.op_ld_r_rr(D, HL),
      0x57 => self.op_ld_r_r(D, A),

      0x58 => self.op_ld_r_r(E, B),
      0x59 => self.op_ld_r_r(E, C),
      0x5A => self.op_ld_r_r(E, D),
      0x5B => self.op_ld_r_r(E, E),
      0x5C => self.op_ld_r_r(E, H),
      0x5D => self.op_ld_r_r(E, L),
      0x5E => self.op_ld_r_rr(E, HL),
      0x5F => self.op_ld_r_r(E, A),

      0x60 => self.op_ld_r_r(H, B),
      0x61 => self.op_ld_r_r(H, C),
      0x62 => self.op_ld_r_r(H, D),
      0x63 => self.op_ld_r_r(H, E),
      0x64 => self.op_ld_r_r(H, H),
      0x65 => self.op_ld_r_r(H, L),
      0x66 => self.op_ld_r_rr(H, HL),
      0x67 => self.op_ld_r_r(H, A),

      0x68 => self.op_ld_r_r(L, B),
      0x69 => self.op_ld_r_r(L, C),
      0x6A => self.op_ld_r_r(L, D),
      0x6B => self.op_ld_r_r(L, E),
      0x6C => self.op_ld_r_r(L, H),
      0x6D => self.op_ld_r_r(L, L),
      0x6E => self.op_ld_r_rr(L, HL),
      0x6F => self.op_ld_r_r(L, A),

      0x70 => self.op_ld_hl_r(B),
      0x71 => self.op_ld_hl_r(C),
      0x72 => self.op_ld_hl_r(D),
      0x73 => self.op_ld_hl_r(E),
      0x74 => self.op_ld_hl_r(H),
      0x75 => self.op_ld_hl_r(L),
      0x76 => self.op_halt(),
      0x77 => self.op_ld_hl_r(A),

      0x78 => self.op_ld_r_r(A, B),
      0x79 => self.op_ld_r_r(A, C),
      0x7A => self.op_ld_r_r(A, D),
      0x7B => self.op_ld_r_r(A, E),
      0x7C => self.op_ld_r_r(A, H),
      0x7D => self.op_ld_r_r(A, L),
      0x7E => self.op_ld_r_rr(A, HL),
      0x7F => self.op_ld_r_r(A, A),

      0x80 => self.op_add_r(B),
      0x81 => self.op_add_r(C),
      0x82 => self.op_add_r(D),
      0x83 => self.op_add_r(E),
      0x84 => self.op_add_r(H),
      0x85 => self.op_add_r(L),
      0x86 => self.op_add_hl(),
      0x87 => self.op_add_r(A),

      0x88 => self.op_adc_r(B),
      0x89 => self.op_adc_r(C),
      0x8A => self.op_adc_r(D),
      0x8B => self.op_adc_r(E),
      0x8C => self.op_adc_r(H),
      0x8D => self.op_adc_r(L),
      0x8E => self.op_adc_hl(),
      0x8F => self.op_adc_r(A),

      0x90 => self.op_sub_r(B),
      0x91 => self.op_sub_r(C),
      0x92 => self.op_sub_r(D),
      0x93 => self.op_sub_r(E),
      0x94 => self.op_sub_r(H),
      0x95 => self.op_sub_r(L),
      0x96 => self.op_sub_hl(),
      0x97 => self.op_sub_r(A),

      0x98 => self.op_sbc_r(B),
      0x99 => self.op_sbc_r(C),
      0x9A => self.op_sbc_r(D),
      0x9B => self.op_sbc_r(E),
      0x9C => self.op_sbc_r(H),
      0x9D => self.op_sbc_r(L),
      0x9E => self.op_sbc_hl(),
      0x9F => self.op_sbc_r(A),

      0xA0 => self.op_and_r(B),
      0xA1 => self.op_and_r(C),
      0xA2 => self.op_and_r(D),
      0xA3 => self.op_and_r(E),
      0xA4 => self.op_and_r(H),
      0xA5 => self.op_and_r(L),
      0xA6 => self.op_and_hl(),
      0xA7 => self.op_and_r(A),

      0xA8 => self.op_xor_r(B),
      0xA9 => self.op_xor_r(C),
      0xAA => self.op_xor_r(D),
      0xAB => self.op_xor_r(E),
      0xAC => self.op_xor_r(H),
      0xAD => self.op_xor_r(L),
      0xAE => self.op_xor_hl(),
      0xAF => self.op_xor_r(A),

      0xB0 => self.op_or_r(B),
      0xB1 => self.op_or_r(C),
      0xB2 => self.op_or_r(D),
      0xB3 => self.op_or_r(E),
      0xB4 => self.op_or_r(H),
      0xB5 => self.op_or_r(L),
      0xB6 => self.op_or_hl(),
      0xB7 => self.op_or_r(A),

      0xB8 => self.op_cp_r(B),
      0xB9 => self.op_cp_r(C),
      0xBA => self.op_cp_r(D),
      0xBB => self.op_cp_r(E),
      0xBC => self.op_cp_r(H),
      0xBD => self.op_cp_r(L),
      0xBE => self.op_cp_hl(),
      0xBF => self.op_cp_r(A),

      0xC0 => self.op_ret_f(FLAG::Z, false),
      0xD0 => self.op_ret_f(FLAG::C, false),
      0xE0 => self.op_ld_ffn_a(),
      0xF0 => self.op_ld_a_ffn(),

      0xC1 => self.op_pop_rr(BC),
      0xD1 => self.op_pop_rr(DE),
      0xE1 => self.op_pop_rr(HL),
      0xF1 => self.op_pop_rr(AF),

      0xC2 => self.op_jp_f_nn(FLAG::Z, false),
      0xD2 => self.op_jp_f_nn(FLAG::C, false),
      0xE2 => self.op_ld_ffc_a(),
      0xF2 => self.op_ld_a_ffc(),

      0xC3 => self.op_jp_nn(),
      0xD3 => self.op_nop(),
      0xE3 => self.op_nop(),
      0xF3 => self.op_di(),

      0xC4 => self.op_call_f_nn(FLAG::Z, false),
      0xD4 => self.op_call_f_nn(FLAG::C, false),
      0xE4 => self.op_nop(),
      0xF4 => self.op_nop(),

      0xC5 => self.op_push_rr(BC),
      0xD5 => self.op_push_rr(DE),
      0xE5 => self.op_push_rr(HL),
      0xF5 => self.op_push_rr(AF),

      0xC6 => self.op_add_n(),
      0xD6 => self.op_sub_n(),
      0xE6 => self.op_and_n(),
      0xF6 => self.op_or_n(),

      0xC7 => self.op_rst(0x00),
      0xD7 => self.op_rst(0x10),
      0xE7 => self.op_rst(0x20),
      0xF7 => self.op_rst(0x30),

      0xC8 => self.op_ret_f(FLAG::Z, true),
      0xD8 => self.op_ret_f(FLAG::C, true),
      0xE8 => self.op_add_sp_dd(),
      0xF8 => self.op_ld_hl_sp_dd(),

      0xC9 => self.op_ret(),
      0xD9 => self.op_reti(),
      0xE9 => self.op_jp_hl(),
      0xF9 => self.op_ld_sp_hl(),

      0xCA => self.op_jp_f_nn(FLAG::Z, true),
      0xDA => self.op_jp_f_nn(FLAG::C, true),
      0xEA => self.op_ld_nn_a(),
      0xFA => self.op_ld_a_nn(),

      0xCB => {
        let cb_opcode = self.read_pc();
        match cb_opcode {
          0x00 => self.op_rlc_r(B),
          0x01 => self.op_rlc_r(C),
          0x02 => self.op_rlc_r(D),
          0x03 => self.op_rlc_r(E),
          0x04 => self.op_rlc_r(H),
          0x05 => self.op_rlc_r(L),
          0x06 => self.op_rlc_hl(),
          0x07 => self.op_rlc_r(A),

          0x08 => self.op_rrc_r(B),
          0x09 => self.op_rrc_r(C),
          0x0A => self.op_rrc_r(D),
          0x0B => self.op_rrc_r(E),
          0x0C => self.op_rrc_r(H),
          0x0D => self.op_rrc_r(L),
          0x0E => self.op_rrc_hl(),
          0x0F => self.op_rrc_r(A),

          0x10 => self.op_rl_r(B),
          0x11 => self.op_rl_r(C),
          0x12 => self.op_rl_r(D),
          0x13 => self.op_rl_r(E),
          0x14 => self.op_rl_r(H),
          0x15 => self.op_rl_r(L),
          0x16 => self.op_rl_hl(),
          0x17 => self.op_rl_r(A),

          0x18 => self.op_rr_r(B),
          0x19 => self.op_rr_r(C),
          0x1A => self.op_rr_r(D),
          0x1B => self.op_rr_r(E),
          0x1C => self.op_rr_r(H),
          0x1D => self.op_rr_r(L),
          0x1E => self.op_rr_hl(),
          0x1F => self.op_rr_r(A),

          0x20 => self.op_sla_r(B),
          0x21 => self.op_sla_r(C),
          0x22 => self.op_sla_r(D),
          0x23 => self.op_sla_r(E),
          0x24 => self.op_sla_r(H),
          0x25 => self.op_sla_r(L),
          0x26 => self.op_sla_hl(),
          0x27 => self.op_sla_r(A),

          0x28 => self.op_sra_r(B),
          0x29 => self.op_sra_r(C),
          0x2A => self.op_sra_r(D),
          0x2B => self.op_sra_r(E),
          0x2C => self.op_sra_r(H),
          0x2D => self.op_sra_r(L),
          0x2E => self.op_sra_hl(),
          0x2F => self.op_sra_r(A),

          0x30 => self.op_swap_r(B),
          0x31 => self.op_swap_r(C),
          0x32 => self.op_swap_r(D),
          0x33 => self.op_swap_r(E),
          0x34 => self.op_swap_r(H),
          0x35 => self.op_swap_r(L),
          0x36 => self.op_swap_hl(),
          0x37 => self.op_swap_r(A),

          0x38 => self.op_srl_r(B),
          0x39 => self.op_srl_r(C),
          0x3A => self.op_srl_r(D),
          0x3B => self.op_srl_r(E),
          0x3C => self.op_srl_r(H),
          0x3D => self.op_srl_r(L),
          0x3E => self.op_srl_hl(),
          0x3F => self.op_srl_r(A),

          0x40 => self.op_bit_n_r(0, B),
          0x41 => self.op_bit_n_r(0, C),
          0x42 => self.op_bit_n_r(0, D),
          0x43 => self.op_bit_n_r(0, E),
          0x44 => self.op_bit_n_r(0, H),
          0x45 => self.op_bit_n_r(0, L),
          0x46 => self.op_bit_n_hl(0),
          0x47 => self.op_bit_n_r(0, A),

          0x48 => self.op_bit_n_r(1, B),
          0x49 => self.op_bit_n_r(1, C),
          0x4A => self.op_bit_n_r(1, D),
          0x4B => self.op_bit_n_r(1, E),
          0x4C => self.op_bit_n_r(1, H),
          0x4D => self.op_bit_n_r(1, L),
          0x4E => self.op_bit_n_hl(1),
          0x4F => self.op_bit_n_r(1, A),

          0x50 => self.op_bit_n_r(2, B),
          0x51 => self.op_bit_n_r(2, C),
          0x52 => self.op_bit_n_r(2, D),
          0x53 => self.op_bit_n_r(2, E),
          0x54 => self.op_bit_n_r(2, H),
          0x55 => self.op_bit_n_r(2, L),
          0x56 => self.op_bit_n_hl(2),
          0x57 => self.op_bit_n_r(2, A),

          0x58 => self.op_bit_n_r(3, B),
          0x59 => self.op_bit_n_r(3, C),
          0x5A => self.op_bit_n_r(3, D),
          0x5B => self.op_bit_n_r(3, E),
          0x5C => self.op_bit_n_r(3, H),
          0x5D => self.op_bit_n_r(3, L),
          0x5E => self.op_bit_n_hl(3),
          0x5F => self.op_bit_n_r(3, A),

          0x60 => self.op_bit_n_r(4, B),
          0x61 => self.op_bit_n_r(4, C),
          0x62 => self.op_bit_n_r(4, D),
          0x63 => self.op_bit_n_r(4, E),
          0x64 => self.op_bit_n_r(4, H),
          0x65 => self.op_bit_n_r(4, L),
          0x66 => self.op_bit_n_hl(4),
          0x67 => self.op_bit_n_r(4, A),

          0x68 => self.op_bit_n_r(5, B),
          0x69 => self.op_bit_n_r(5, C),
          0x6A => self.op_bit_n_r(5, D),
          0x6B => self.op_bit_n_r(5, E),
          0x6C => self.op_bit_n_r(5, H),
          0x6D => self.op_bit_n_r(5, L),
          0x6E => self.op_bit_n_hl(5),
          0x6F => self.op_bit_n_r(5, A),

          0x70 => self.op_bit_n_r(6, B),
          0x71 => self.op_bit_n_r(6, C),
          0x72 => self.op_bit_n_r(6, D),
          0x73 => self.op_bit_n_r(6, E),
          0x74 => self.op_bit_n_r(6, H),
          0x75 => self.op_bit_n_r(6, L),
          0x76 => self.op_bit_n_hl(6),
          0x77 => self.op_bit_n_r(6, A),

          0x78 => self.op_bit_n_r(7, B),
          0x79 => self.op_bit_n_r(7, C),
          0x7A => self.op_bit_n_r(7, D),
          0x7B => self.op_bit_n_r(7, E),
          0x7C => self.op_bit_n_r(7, H),
          0x7D => self.op_bit_n_r(7, L),
          0x7E => self.op_bit_n_hl(7),
          0x7F => self.op_bit_n_r(7, A),

          0x80 => self.op_set_n_r(0, B),
          0x81 => self.op_set_n_r(0, C),
          0x82 => self.op_set_n_r(0, D),
          0x83 => self.op_set_n_r(0, E),
          0x84 => self.op_set_n_r(0, H),
          0x85 => self.op_set_n_r(0, L),
          0x86 => self.op_set_n_hl(0),
          0x87 => self.op_set_n_r(0, A),

          0x88 => self.op_set_n_r(1, B),
          0x89 => self.op_set_n_r(1, C),
          0x8A => self.op_set_n_r(1, D),
          0x8B => self.op_set_n_r(1, E),
          0x8C => self.op_set_n_r(1, H),
          0x8D => self.op_set_n_r(1, L),
          0x8E => self.op_set_n_hl(1),
          0x8F => self.op_set_n_r(1, A),

          0x90 => self.op_set_n_r(2, B),
          0x91 => self.op_set_n_r(2, C),
          0x92 => self.op_set_n_r(2, D),
          0x93 => self.op_set_n_r(2, E),
          0x94 => self.op_set_n_r(2, H),
          0x95 => self.op_set_n_r(2, L),
          0x96 => self.op_set_n_hl(2),
          0x97 => self.op_set_n_r(2, A),

          0x98 => self.op_set_n_r(3, B),
          0x99 => self.op_set_n_r(3, C),
          0x9A => self.op_set_n_r(3, D),
          0x9B => self.op_set_n_r(3, E),
          0x9C => self.op_set_n_r(3, H),
          0x9D => self.op_set_n_r(3, L),
          0x9E => self.op_set_n_hl(3),
          0x9F => self.op_set_n_r(3, A),

          0xA0 => self.op_set_n_r(4, B),
          0xA1 => self.op_set_n_r(4, C),
          0xA2 => self.op_set_n_r(4, D),
          0xA3 => self.op_set_n_r(4, E),
          0xA4 => self.op_set_n_r(4, H),
          0xA5 => self.op_set_n_r(4, L),
          0xA6 => self.op_set_n_hl(4),
          0xA7 => self.op_set_n_r(4, A),

          0xA8 => self.op_set_n_r(5, B),
          0xA9 => self.op_set_n_r(5, C),
          0xAA => self.op_set_n_r(5, D),
          0xAB => self.op_set_n_r(5, E),
          0xAC => self.op_set_n_r(5, H),
          0xAD => self.op_set_n_r(5, L),
          0xAE => self.op_set_n_hl(5),
          0xAF => self.op_set_n_r(5, A),

          0xB0 => self.op_set_n_r(6, B),
          0xB1 => self.op_set_n_r(6, C),
          0xB2 => self.op_set_n_r(6, D),
          0xB3 => self.op_set_n_r(6, E),
          0xB4 => self.op_set_n_r(6, H),
          0xB5 => self.op_set_n_r(6, L),
          0xB6 => self.op_set_n_hl(6),
          0xB7 => self.op_set_n_r(6, A),

          0xB8 => self.op_set_n_r(7, B),
          0xB9 => self.op_set_n_r(7, C),
          0xBA => self.op_set_n_r(7, D),
          0xBB => self.op_set_n_r(7, E),
          0xBC => self.op_set_n_r(7, H),
          0xBD => self.op_set_n_r(7, L),
          0xBE => self.op_set_n_hl(7),
          0xBF => self.op_set_n_r(7, A),

          0xC0 => self.op_res_n_r(0, B),
          0xC1 => self.op_res_n_r(0, C),
          0xC2 => self.op_res_n_r(0, D),
          0xC3 => self.op_res_n_r(0, E),
          0xC4 => self.op_res_n_r(0, H),
          0xC5 => self.op_res_n_r(0, L),
          0xC6 => self.op_res_n_hl(0),
          0xC7 => self.op_res_n_r(0, A),

          0xC8 => self.op_res_n_r(1, B),
          0xC9 => self.op_res_n_r(1, C),
          0xCA => self.op_res_n_r(1, D),
          0xCB => self.op_res_n_r(1, E),
          0xCC => self.op_res_n_r(1, H),
          0xCD => self.op_res_n_r(1, L),
          0xCE => self.op_res_n_hl(1),
          0xCF => self.op_res_n_r(1, A),

          0xD0 => self.op_res_n_r(2, B),
          0xD1 => self.op_res_n_r(2, C),
          0xD2 => self.op_res_n_r(2, D),
          0xD3 => self.op_res_n_r(2, E),
          0xD4 => self.op_res_n_r(2, H),
          0xD5 => self.op_res_n_r(2, L),
          0xD6 => self.op_res_n_hl(2),
          0xD7 => self.op_res_n_r(2, A),

          0xD8 => self.op_res_n_r(3, B),
          0xD9 => self.op_res_n_r(3, C),
          0xDA => self.op_res_n_r(3, D),
          0xDB => self.op_res_n_r(3, E),
          0xDC => self.op_res_n_r(3, H),
          0xDD => self.op_res_n_r(3, L),
          0xDE => self.op_res_n_hl(3),
          0xDF => self.op_res_n_r(3, A),

          0xE0 => self.op_res_n_r(4, B),
          0xE1 => self.op_res_n_r(4, C),
          0xE2 => self.op_res_n_r(4, D),
          0xE3 => self.op_res_n_r(4, E),
          0xE4 => self.op_res_n_r(4, H),
          0xE5 => self.op_res_n_r(4, L),
          0xE6 => self.op_res_n_hl(4),
          0xE7 => self.op_res_n_r(4, A),

          0xE8 => self.op_res_n_r(5, B),
          0xE9 => self.op_res_n_r(5, C),
          0xEA => self.op_res_n_r(5, D),
          0xEB => self.op_res_n_r(5, E),
          0xEC => self.op_res_n_r(5, H),
          0xED => self.op_res_n_r(5, L),
          0xEE => self.op_res_n_hl(5),
          0xEF => self.op_res_n_r(5, A),

          0xF0 => self.op_res_n_r(6, B),
          0xF1 => self.op_res_n_r(6, C),
          0xF2 => self.op_res_n_r(6, D),
          0xF3 => self.op_res_n_r(6, E),
          0xF4 => self.op_res_n_r(6, H),
          0xF5 => self.op_res_n_r(6, L),
          0xF6 => self.op_res_n_hl(6),
          0xF7 => self.op_res_n_r(6, A),

          0xF8 => self.op_res_n_r(7, B),
          0xF9 => self.op_res_n_r(7, C),
          0xFA => self.op_res_n_r(7, D),
          0xFB => self.op_res_n_r(7, E),
          0xFC => self.op_res_n_r(7, H),
          0xFD => self.op_res_n_r(7, L),
          0xFE => self.op_res_n_hl(7),
          0xFF => self.op_res_n_r(7, A),

          _ => unreachable!(),
        }
      },
      0xDB => self.op_nop(),
      0xEB => self.op_nop(),
      0xFB => self.op_ei(),

      0xCC => self.op_call_f_nn(FLAG::Z, true),
      0xDC => self.op_call_f_nn(FLAG::C, true),
      0xEC => self.op_nop(),
      0xFC => self.op_nop(),

      0xCD => self.op_call_nn(),
      0xDD => self.op_nop(),
      0xED => self.op_nop(),
      0xFD => self.op_nop(),

      0xCE => self.op_adc_n(),
      0xDE => self.op_sbc_n(),
      0xEE => self.op_xor_n(),
      0xFE => self.op_cp_n(),

      0xCF => self.op_rst(0x08),
      0xDF => self.op_rst(0x18),
      0xEF => self.op_rst(0x28),
      0xFF => self.op_rst(0x38),

      _ => unreachable!(),
    }
  }

  fn decode(&self, opcode: u8) -> String {
    // Get arguments without incrementing PC
    let arg = self.read(self.rr(PC));
    let arg16 = self.read_16le(self.rr(PC));

    match opcode {
      0x00 => format!("NOP"),
      0x10 => format!("STOP"),
      0x20 => format!("JR NZ,{}", arg as i8),
      0x30 => format!("JR NC,{}", arg as i8),

      0x01 => format!("LD BC,{:x}", arg16),
      0x11 => format!("LD DE,{:x}", arg16),
      0x21 => format!("LD HL,{:x}", arg16),
      0x31 => format!("LD SP,{:x}", arg16),

      0x02 => format!("LD BC,A"),
      0x12 => format!("LD DE,A"),
      0x22 => format!("LD (HL+),A"),
      0x32 => format!("LD (HL-),A"),

      0x03 => format!("INC BC"),
      0x13 => format!("INC DE"),
      0x23 => format!("INC HL"),
      0x33 => format!("INC SP"),

      0x04 => format!("INC B"),
      0x14 => format!("INC D"),
      0x24 => format!("INC H"),
      0x34 => format!("INC (HL)"),

      0x05 => format!("DEC B"),
      0x15 => format!("DEC D"),
      0x25 => format!("DEC H"),
      0x35 => format!("DEC (HL)"),

      0x06 => format!("LD B,{:x}", arg),
      0x16 => format!("LD D,{:x}", arg),
      0x26 => format!("LD H,{:x}", arg),
      0x36 => format!("LD (HL),{:x}", arg),

      0x07 => format!("RLCA"),
      0x17 => format!("RLA"),
      0x27 => format!("DAA"),
      0x37 => format!("SCF"),

      0x08 => format!("LD ({:x}),SP", arg16),
      0x18 => format!("JR {}", arg as i8),
      0x28 => format!("JR Z,{}", arg as i8),
      0x38 => format!("JR C,{}", arg as i8),

      0x09 => format!("ADD HL,BC"),
      0x19 => format!("ADD HL,DE"),
      0x29 => format!("ADD HL,HL"),
      0x39 => format!("ADD HL,SP"),

      0x0A => format!("LD A,(BC)"),
      0x1A => format!("LD A,(DE)"),
      0x2A => format!("LD A,(HL+)"),
      0x3A => format!("LD A,(HL-)"),

      0x0B => format!("DEC BC"),
      0x1B => format!("DEC DE"),
      0x2B => format!("DEC HL"),
      0x3B => format!("DEC SP"),

      0x0C => format!("INC C"),
      0x1C => format!("INC E"),
      0x2C => format!("INC L"),
      0x3C => format!("INC A"),

      0x0D => format!("DEC C"),
      0x1D => format!("DEC E"),
      0x2D => format!("DEC L"),
      0x3D => format!("DEC A"),

      0x0E => format!("LD C,{:x}", arg),
      0x1E => format!("LD E,{:x}", arg),
      0x2E => format!("LD L,{:x}", arg),
      0x3E => format!("LD A,{:x}", arg),

      0x0F => format!("RRCA"),
      0x1F => format!("RRA"),
      0x2F => format!("CPL"),
      0x3F => format!("CCF"),

      0x40 => format!("LD B,B"),
      0x41 => format!("LD B,C"),
      0x42 => format!("LD B,D"),
      0x43 => format!("LD B,E"),
      0x44 => format!("LD B,H"),
      0x45 => format!("LD B,L"),
      0x46 => format!("LD B,(HL)"),
      0x47 => format!("LD B,A"),

      0x48 => format!("LD C,B"),
      0x49 => format!("LD C,C"),
      0x4A => format!("LD C,D"),
      0x4B => format!("LD C,E"),
      0x4C => format!("LD C,H"),
      0x4D => format!("LD C,L"),
      0x4E => format!("LD C,(HL)"),
      0x4F => format!("LD C,A"),

      0x50 => format!("LD D,B"),
      0x51 => format!("LD D,C"),
      0x52 => format!("LD D,D"),
      0x53 => format!("LD D,E"),
      0x54 => format!("LD D,H"),
      0x55 => format!("LD D,L"),
      0x56 => format!("LD D,(HL)"),
      0x57 => format!("LD D,A"),

      0x58 => format!("LD E,B"),
      0x59 => format!("LD E,C"),
      0x5A => format!("LD E,D"),
      0x5B => format!("LD E,E"),
      0x5C => format!("LD E,H"),
      0x5D => format!("LD E,L"),
      0x5E => format!("LD E,(HL)"),
      0x5F => format!("LD E,A"),

      0x60 => format!("LD H,B"),
      0x61 => format!("LD H,C"),
      0x62 => format!("LD H,D"),
      0x63 => format!("LD H,E"),
      0x64 => format!("LD H,H"),
      0x65 => format!("LD H,L"),
      0x66 => format!("LD H,(HL)"),
      0x67 => format!("LD H,A"),

      0x68 => format!("LD L,B"),
      0x69 => format!("LD L,C"),
      0x6A => format!("LD L,D"),
      0x6B => format!("LD L,E"),
      0x6C => format!("LD L,H"),
      0x6D => format!("LD L,L"),
      0x6E => format!("LD L,(HL)"),
      0x6F => format!("LD L,A"),

      0x70 => format!("LD (HL),B"),
      0x71 => format!("LD (HL),C"),
      0x72 => format!("LD (HL),D"),
      0x73 => format!("LD (HL),E"),
      0x74 => format!("LD (HL),H"),
      0x75 => format!("LD (HL),L"),
      0x76 => format!("HALT"),
      0x77 => format!("LD (HL),A"),

      0x78 => format!("LD A,B"),
      0x79 => format!("LD A,C"),
      0x7A => format!("LD A,D"),
      0x7B => format!("LD A,E"),
      0x7C => format!("LD A,H"),
      0x7D => format!("LD A,L"),
      0x7E => format!("LD A,(HL)"),
      0x7F => format!("LD A,A"),

      0x80 => format!("ADD A,B"),
      0x81 => format!("ADD A,C"),
      0x82 => format!("ADD A,D"),
      0x83 => format!("ADD A,E"),
      0x84 => format!("ADD A,H"),
      0x85 => format!("ADD A,L"),
      0x86 => format!("ADD A,(HL)"),
      0x87 => format!("ADD A,A"),

      0x88 => format!("ADC A,B"),
      0x89 => format!("ADC A,C"),
      0x8A => format!("ADC A,D"),
      0x8B => format!("ADC A,E"),
      0x8C => format!("ADC A,H"),
      0x8D => format!("ADC A,L"),
      0x8E => format!("ADC A,(HL)"),
      0x8F => format!("ADC A,A"),

      0x90 => format!("SUB A,B"),
      0x91 => format!("SUB A,C"),
      0x92 => format!("SUB A,D"),
      0x93 => format!("SUB A,E"),
      0x94 => format!("SUB A,H"),
      0x95 => format!("SUB A,L"),
      0x96 => format!("SUB A,(HL)"),
      0x97 => format!("SUB A,A"),

      0x98 => format!("SBC A,B"),
      0x99 => format!("SBC A,C"),
      0x9A => format!("SBC A,D"),
      0x9B => format!("SBC A,E"),
      0x9C => format!("SBC A,H"),
      0x9D => format!("SBC A,L"),
      0x9E => format!("SBC A,(HL)"),
      0x9F => format!("SBC A,A"),

      0xA0 => format!("AND B"),
      0xA1 => format!("AND C"),
      0xA2 => format!("AND D"),
      0xA3 => format!("AND E"),
      0xA4 => format!("AND H"),
      0xA5 => format!("AND L"),
      0xA6 => format!("AND (HL)"),
      0xA7 => format!("AND A"),

      0xA8 => format!("XOR B"),
      0xA9 => format!("XOR C"),
      0xAA => format!("XOR D"),
      0xAB => format!("XOR E"),
      0xAC => format!("XOR H"),
      0xAD => format!("XOR L"),
      0xAE => format!("XOR (HL)"),
      0xAF => format!("XOR A"),

      0xB0 => format!("OR B"),
      0xB1 => format!("OR C"),
      0xB2 => format!("OR D"),
      0xB3 => format!("OR E"),
      0xB4 => format!("OR H"),
      0xB5 => format!("OR L"),
      0xB6 => format!("OR (HL)"),
      0xB7 => format!("OR A"),

      0xB8 => format!("CP B"),
      0xB9 => format!("CP C"),
      0xBA => format!("CP D"),
      0xBB => format!("CP E"),
      0xBC => format!("CP H"),
      0xBD => format!("CP L"),
      0xBE => format!("CP (HL)"),
      0xBF => format!("CP A"),

      0xC0 => format!("RET NZ"),
      0xD0 => format!("RET NC"),
      0xE0 => format!("LDH ({:x}),A", 0xFF00 | (arg as u16)),
      0xF0 => format!("LDH A,({:x})", 0xFF00 | (arg as u16)),

      0xC1 => format!("POP BC"),
      0xD1 => format!("POP DE"),
      0xE1 => format!("POP HL"),
      0xF1 => format!("POP AF"),

      0xC2 => format!("JP NZ,{:x}", arg16),
      0xD2 => format!("JP NC,{:x}", arg16),
      0xE2 => format!("LD (C),A"),
      0xF2 => format!("LD A,(C)"),

      0xC3 => format!("JP {:x}", arg16),
      0xD3 => format!("NOP"),
      0xE3 => format!("NOP"),
      0xF3 => format!("DI"),

      0xC4 => format!("CALL NZ,{:x}", arg16),
      0xD4 => format!("CALL NC,{:x}", arg16),
      0xE4 => format!("NOP"),
      0xF4 => format!("NOP"),

      0xC5 => format!("PUSH BC"),
      0xD5 => format!("PUSH DE"),
      0xE5 => format!("PUSH HL"),
      0xF5 => format!("PUSH AF"),

      0xC6 => format!("ADD A,{:x}", arg),
      0xD6 => format!("SUB A,{:x}", arg),
      0xE6 => format!("AND A,{:x}", arg),
      0xF6 => format!("OR A,{:x}", arg),

      0xC7 => format!("RST 00H"),
      0xD7 => format!("RST 10H"),
      0xE7 => format!("RST 20H"),
      0xF7 => format!("RST 30H"),

      0xC8 => format!("RET Z"),
      0xD8 => format!("RET C"),
      0xE8 => format!("ADD SP,{}", arg as i8),
      0xF8 => format!("LD HL,SP+{}", arg as i8),

      0xC9 => format!("RET"),
      0xD9 => format!("RETI"),
      0xE9 => format!("JP (HL)"),
      0xF9 => format!("LD SP,HL"),

      0xCA => format!("JP Z,{:x}", arg16),
      0xDA => format!("JP C,{:x}", arg16),
      0xEA => format!("LD ({:x}),A", arg16),
      0xFA => format!("LD A,({:x})", arg16),

      0xCB => {
        match arg {
          0x00 => format!("RLC B"),
          0x01 => format!("RLC C"),
          0x02 => format!("RLC D"),
          0x03 => format!("RLC E"),
          0x04 => format!("RLC H"),
          0x05 => format!("RLC L"),
          0x06 => format!("RLC (HL)"),
          0x07 => format!("RLC A"),

          0x08 => format!("RRC B"),
          0x09 => format!("RRC C"),
          0x0A => format!("RRC D"),
          0x0B => format!("RRC E"),
          0x0C => format!("RLC H"),
          0x0D => format!("RRC L"),
          0x0E => format!("RRC (HL)"),
          0x0F => format!("RRC A"),

          0x10 => format!("RL B"),
          0x11 => format!("RL C"),
          0x12 => format!("RL D"),
          0x13 => format!("RL E"),
          0x14 => format!("RL H"),
          0x15 => format!("RL L"),
          0x16 => format!("RL (HL)"),
          0x17 => format!("RL A"),

          0x18 => format!("RR B"),
          0x19 => format!("RR C"),
          0x1A => format!("RR D"),
          0x1B => format!("RR E"),
          0x1C => format!("RL H"),
          0x1D => format!("RR L"),
          0x1E => format!("RR (HL)"),
          0x1F => format!("RR A"),

          0x20 => format!("SLA B"),
          0x21 => format!("SLA C"),
          0x22 => format!("SLA D"),
          0x23 => format!("SLA E"),
          0x24 => format!("SLA H"),
          0x25 => format!("SLA L"),
          0x26 => format!("SLA (HL)"),
          0x27 => format!("SLA A"),

          0x28 => format!("SRA B"),
          0x29 => format!("SRA C"),
          0x2A => format!("SRA D"),
          0x2B => format!("SRA E"),
          0x2C => format!("SRA H"),
          0x2D => format!("SRA L"),
          0x2E => format!("SRA (HL)"),
          0x2F => format!("SRA A"),

          0x30 => format!("SWAP B"),
          0x31 => format!("SWAP C"),
          0x32 => format!("SWAP D"),
          0x33 => format!("SWAP E"),
          0x34 => format!("SWAP H"),
          0x35 => format!("SWAP L"),
          0x36 => format!("SWAP (HL)"),
          0x37 => format!("SWAP A"),

          0x38 => format!("SRL B"),
          0x39 => format!("SRL C"),
          0x3A => format!("SRL D"),
          0x3B => format!("SRL E"),
          0x3C => format!("SRL H"),
          0x3D => format!("SRL L"),
          0x3E => format!("SRL (HL)"),
          0x3F => format!("SRL A"),

          0x40 => format!("BIT 0,B"),
          0x41 => format!("BIT 0,C"),
          0x42 => format!("BIT 0,D"),
          0x43 => format!("BIT 0,E"),
          0x44 => format!("BIT 0,H"),
          0x45 => format!("BIT 0,L"),
          0x46 => format!("BIT 0,(HL)"),
          0x47 => format!("BIT 0,A"),

          0x48 => format!("BIT 1,B"),
          0x49 => format!("BIT 1,C"),
          0x4A => format!("BIT 1,D"),
          0x4B => format!("BIT 1,E"),
          0x4C => format!("BIT 1,H"),
          0x4D => format!("BIT 1,L"),
          0x4E => format!("BIT 1,(HL)"),
          0x4F => format!("BIT 1,A"),

          0x50 => format!("BIT 2,B"),
          0x51 => format!("BIT 2,C"),
          0x52 => format!("BIT 2,D"),
          0x53 => format!("BIT 2,E"),
          0x54 => format!("BIT 2,H"),
          0x55 => format!("BIT 2,L"),
          0x56 => format!("BIT 2,(HL)"),
          0x57 => format!("BIT 2,A"),

          0x58 => format!("BIT 3,B"),
          0x59 => format!("BIT 3,C"),
          0x5A => format!("BIT 3,D"),
          0x5B => format!("BIT 3,E"),
          0x5C => format!("BIT 3,H"),
          0x5D => format!("BIT 3,L"),
          0x5E => format!("BIT 3,(HL)"),
          0x5F => format!("BIT 3,A"),

          0x60 => format!("BIT 4,B"),
          0x61 => format!("BIT 4,C"),
          0x62 => format!("BIT 4,D"),
          0x63 => format!("BIT 4,E"),
          0x64 => format!("BIT 4,H"),
          0x65 => format!("BIT 4,L"),
          0x66 => format!("BIT 4,(HL)"),
          0x67 => format!("BIT 4,A"),

          0x68 => format!("BIT 5,B"),
          0x69 => format!("BIT 5,C"),
          0x6A => format!("BIT 5,D"),
          0x6B => format!("BIT 5,E"),
          0x6C => format!("BIT 5,H"),
          0x6D => format!("BIT 5,L"),
          0x6E => format!("BIT 5,(HL)"),
          0x6F => format!("BIT 5,A"),

          0x70 => format!("BIT 6,B"),
          0x71 => format!("BIT 6,C"),
          0x72 => format!("BIT 6,D"),
          0x73 => format!("BIT 6,E"),
          0x74 => format!("BIT 6,H"),
          0x75 => format!("BIT 6,L"),
          0x76 => format!("BIT 6,(HL)"),
          0x77 => format!("BIT 6,A"),

          0x78 => format!("BIT 7,B"),
          0x79 => format!("BIT 7,C"),
          0x7A => format!("BIT 7,D"),
          0x7B => format!("BIT 7,E"),
          0x7C => format!("BIT 7,H"),
          0x7D => format!("BIT 7,L"),
          0x7E => format!("BIT 7,(HL)"),
          0x7F => format!("BIT 7,A"),

          0x80 => format!("RES 0,B"),
          0x81 => format!("RES 0,C"),
          0x82 => format!("RES 0,D"),
          0x83 => format!("RES 0,E"),
          0x84 => format!("RES 0,H"),
          0x85 => format!("RES 0,L"),
          0x86 => format!("RES 0,(HL)"),
          0x87 => format!("RES 0,A"),

          0x88 => format!("RES 1,B"),
          0x89 => format!("RES 1,C"),
          0x8A => format!("RES 1,D"),
          0x8B => format!("RES 1,E"),
          0x8C => format!("RES 1,H"),
          0x8D => format!("RES 1,L"),
          0x8E => format!("RES 1,(HL)"),
          0x8F => format!("RES 1,A"),

          0x90 => format!("RES 2,B"),
          0x91 => format!("RES 2,C"),
          0x92 => format!("RES 2,D"),
          0x93 => format!("RES 2,E"),
          0x94 => format!("RES 2,H"),
          0x95 => format!("RES 2,L"),
          0x96 => format!("RES 2,(HL)"),
          0x97 => format!("RES 2,A"),

          0x98 => format!("RES 3,B"),
          0x99 => format!("RES 3,C"),
          0x9A => format!("RES 3,D"),
          0x9B => format!("RES 3,E"),
          0x9C => format!("RES 3,H"),
          0x9D => format!("RES 3,L"),
          0x9E => format!("RES 3,(HL)"),
          0x9F => format!("RES 3,A"),

          0xA0 => format!("RES 4,B"),
          0xA1 => format!("RES 4,C"),
          0xA2 => format!("RES 4,D"),
          0xA3 => format!("RES 4,E"),
          0xA4 => format!("RES 4,H"),
          0xA5 => format!("RES 4,L"),
          0xA6 => format!("RES 4,(HL)"),
          0xA7 => format!("RES 4,A"),

          0xA8 => format!("RES 5,B"),
          0xA9 => format!("RES 5,C"),
          0xAA => format!("RES 5,D"),
          0xAB => format!("RES 5,E"),
          0xAC => format!("RES 5,H"),
          0xAD => format!("RES 5,L"),
          0xAE => format!("RES 5,(HL)"),
          0xAF => format!("RES 5,A"),

          0xB0 => format!("RES 6,B"),
          0xB1 => format!("RES 6,C"),
          0xB2 => format!("RES 6,D"),
          0xB3 => format!("RES 6,E"),
          0xB4 => format!("RES 6,H"),
          0xB5 => format!("RES 6,L"),
          0xB6 => format!("RES 6,(HL)"),
          0xB7 => format!("RES 6,A"),

          0xB8 => format!("RES 7,B"),
          0xB9 => format!("RES 7,C"),
          0xBA => format!("RES 7,D"),
          0xBB => format!("RES 7,E"),
          0xBC => format!("RES 7,H"),
          0xBD => format!("RES 7,L"),
          0xBE => format!("RES 7,(HL)"),
          0xBF => format!("RES 7,A"),

          0xC0 => format!("SET 0,B"),
          0xC1 => format!("SET 0,C"),
          0xC2 => format!("SET 0,D"),
          0xC3 => format!("SET 0,E"),
          0xC4 => format!("SET 0,H"),
          0xC5 => format!("SET 0,L"),
          0xC6 => format!("SET 0,(HL)"),
          0xC7 => format!("SET 0,A"),

          0xC8 => format!("SET 1,B"),
          0xC9 => format!("SET 1,C"),
          0xCA => format!("SET 1,D"),
          0xCB => format!("SET 1,E"),
          0xCC => format!("SET 1,H"),
          0xCD => format!("SET 1,L"),
          0xCE => format!("SET 1,(HL)"),
          0xCF => format!("SET 1,A"),

          0xD0 => format!("SET 2,B"),
          0xD1 => format!("SET 2,C"),
          0xD2 => format!("SET 2,D"),
          0xD3 => format!("SET 2,E"),
          0xD4 => format!("SET 2,H"),
          0xD5 => format!("SET 2,L"),
          0xD6 => format!("SET 2,(HL)"),
          0xD7 => format!("SET 2,A"),

          0xD8 => format!("SET 3,B"),
          0xD9 => format!("SET 3,C"),
          0xDA => format!("SET 3,D"),
          0xDB => format!("SET 3,E"),
          0xDC => format!("SET 3,H"),
          0xDD => format!("SET 3,L"),
          0xDE => format!("SET 3,(HL)"),
          0xDF => format!("SET 3,A"),

          0xE0 => format!("SET 4,B"),
          0xE1 => format!("SET 4,C"),
          0xE2 => format!("SET 4,D"),
          0xE3 => format!("SET 4,E"),
          0xE4 => format!("SET 4,H"),
          0xE5 => format!("SET 4,L"),
          0xE6 => format!("SET 4,(HL)"),
          0xE7 => format!("SET 4,A"),

          0xE8 => format!("SET 5,B"),
          0xE9 => format!("SET 5,C"),
          0xEA => format!("SET 5,D"),
          0xEB => format!("SET 5,E"),
          0xEC => format!("SET 5,H"),
          0xED => format!("SET 5,L"),
          0xEE => format!("SET 5,(HL)"),
          0xEF => format!("SET 5,A"),

          0xF0 => format!("SET 6,B"),
          0xF1 => format!("SET 6,C"),
          0xF2 => format!("SET 6,D"),
          0xF3 => format!("SET 6,E"),
          0xF4 => format!("SET 6,H"),
          0xF5 => format!("SET 6,L"),
          0xF6 => format!("SET 6,(HL)"),
          0xF7 => format!("SET 6,A"),

          0xF8 => format!("SET 7,B"),
          0xF9 => format!("SET 7,C"),
          0xFA => format!("SET 7,D"),
          0xFB => format!("SET 7,E"),
          0xFC => format!("SET 7,H"),
          0xFD => format!("SET 7,L"),
          0xFE => format!("SET 7,(HL)"),
          0xFF => format!("SET 7,A"),

          _ => unreachable!(),
        }
      },
      0xDB => format!("NOP"),
      0xEB => format!("NOP"),
      0xFB => format!("EI"),

      0xCC => format!("CALL Z,{:x}", arg16),
      0xDC => format!("CALL C,{:x}", arg16),
      0xEC => format!("NOP"),
      0xFC => format!("NOP"),

      0xCD => format!("CALL {:x}", arg16),
      0xDD => format!("NOP"),
      0xED => format!("NOP"),
      0xFD => format!("NOP"),

      0xCE => format!("ADC A,{:x}", arg),
      0xDE => format!("SBC A,{:x}", arg),
      0xEE => format!("XOR {:x}", arg),
      0xFE => format!("CP {:x}", arg),

      0xCF => format!("RST 08H"),
      0xDF => format!("RST 18H"),
      0xEF => format!("RST 28H"),
      0xFF => format!("RST 38H"),

      _ => unreachable!(),
    }

  }
}
