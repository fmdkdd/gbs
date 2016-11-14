use cpu::registers::{Registers, R8, R16, FLAG};
use cpu::registers::R8::*;
use cpu::registers::R16::*;
use utils::{from_u16, to_u16};

const RAM_LENGTH : usize = 0x10000;

const ASCII : [char; 256] = [
  ' ', ' ', ' ', ' ', ' ', ' ', ' ',
  ' ', ' ', ' ', ' ', ' ', ' ', ' ',
  ' ', ' ', ' ', ' ', ' ', ' ', ' ',
  ' ', ' ', ' ', ' ', ' ', ' ', ' ',
  ' ', ' ', ' ', ' ',

  ' ', '!', '"', '$', '%', '&', '\'',
  '(', ')', '*', '+', ',', '-', '.',
  '/', '0', '1', '2', '3', '4', '5',
  '6', '7', '8', '9', ':', ';', '<',
  '=', '>', '?', '@', 'A', 'B', 'C',
  'D', 'E', 'F', 'G', 'H', 'I', 'J',
  'K', 'L', 'M', 'N', 'O', 'P', 'Q',
  'R', 'S', 'T', 'U', 'V', 'W', 'X',
  'Y', 'Z', '[', '\\', ']', '^', '_',
  '`', 'a', 'b', 'c', 'd', 'e', 'f',
  'g', 'h', 'i', 'j', 'k', 'l', 'm',
  'n', 'o', 'p', 'q', 'r', 's', 't',
  'u', 'v', 'w', 'x', 'y', 'z', '{',
  '}', '~', ' ',

  ' ', ' ', ' ', ' ', ' ', ' ', ' ',
  ' ', ' ', ' ', ' ', ' ', ' ', ' ',
  ' ', ' ', ' ', ' ', ' ', ' ', ' ',
  ' ', ' ', ' ', ' ', ' ', ' ', ' ',
  ' ', ' ', ' ', ' ',

  ' ', '!', '"', '$', '%', '&', '\'',
  '(', ')', '*', '+', ',', '-', '.',
  '/', '0', '1', '2', '3', '4', '5',
  '6', '7', '8', '9', ':', ';', '<',
  '=', '>', '?', '@', 'A', 'B', 'C',
  'D', 'E', 'F', 'G', 'H', 'I', 'J',
  'K', 'L', 'M', 'N', 'O', 'P', 'Q',
  'R', 'S', 'T', 'U', 'V', 'W', 'X',
  'Y', 'Z', '[', '\\', ']', '^', '_',
  '`', 'a', 'b', 'c', 'd', 'e', 'f',
  'g', 'h', 'i', 'j', 'k', 'l', 'm',
  'n', 'o', 'p', 'q', 'r', 's', 't',
  'u', 'v', 'w', 'x', 'y', 'z', '{',
  '}', '~', ' ',

  ' ', ' ', ' ', ' '
];

pub struct Cpu {
  r: Registers,
  ram: [u8; RAM_LENGTH],
  pub ime: u8,
}

impl Cpu {
  pub fn new() -> Cpu {
    Cpu {
      r: Registers::new(),
      ram: [0; RAM_LENGTH],
      ime: 0,
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
    // TODO
    self.r_set(A, 0x00);
    self.r_set(B, 0x00);
    self.r_set(C, 0x13);
    self.r_set(D, 0x00);
    self.r_set(E, 0xD8);
    self.r_set(H, 0x01);
    self.r_set(L, 0x4D);
    self.r_set(F, 0x01);
    self.rr_set(PC, 0x0100);
    self.rr_set(SP, 0xFFFE);
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

  pub fn read_pc(&mut self) -> u8 {
    let mut pc = self.rr(PC);
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
    self.ram[addr as usize]
  }

  pub fn read_16le(&self, addr: u16) -> u16 {
    let l = self.ram[addr as usize];
    let h = self.ram[(addr.wrapping_add(1)) as usize];
    to_u16(h, l)
  }

  pub fn write(&mut self, addr: u16, x: u8) {
    if addr == 0xFF01 {
      println!("{:x} {}", x, ASCII[x as usize]);
    }

    self.ram[addr as usize] = x;
  }

  pub fn write_16le(&mut self, addr: u16, x: u16) {
    let (h, l) = from_u16(x);
    self.ram[addr as usize] = l;
    self.ram[(addr.wrapping_add(1)) as usize] = h;
  }

  pub fn tile_table(&self) -> &[u8] {
    &self.ram[0x8000..0x8FFF]
  }

  // Run the next instruction
  pub fn step(&mut self) -> u8 {
    let opcode = self.read_pc();
    match opcode {
      // Following the table at
      // http://www.pastraiser.com/cpu/gameboy/gameboy_opcodes.html
      // Since none of the docs I found will actually list the exact opcodes,
      // just the mnemonics.

      0x00 => self.op_nop(),
      0x10 => self.op_stop(),
      // 0x20 => self.op_jr(Z),
      // 0x30 => self.op_jr(C),

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
      // 0x18 => jr!(dd),
      // 0x28 => jr!(z),
      // 0x38 => jr!(c),

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

      0x68 => self.op_ld_r_r(H, B),
      0x69 => self.op_ld_r_r(H, C),
      0x6A => self.op_ld_r_r(H, D),
      0x6B => self.op_ld_r_r(H, E),
      0x6C => self.op_ld_r_r(H, H),
      0x6D => self.op_ld_r_r(H, L),
      0x6E => self.op_ld_r_rr(H, HL),
      0x6F => self.op_ld_r_r(H, A),

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

      // 0xC0 => ret!(nz),
      // 0xD0 => ret!(nc),
      0xE0 => self.op_ld_ffn_a(),
      0xF0 => self.op_ld_a_ffn(),

      0xC1 => self.op_pop_rr(BC),
      0xD1 => self.op_pop_rr(DE),
      0xE1 => self.op_pop_rr(HL),
      0xF1 => self.op_pop_rr(AF),

      // 0xC2 => jp!(nz),
      // 0xD2 => jp!(nc),
      0xE2 => self.op_ld_ffc_a(),
      0xF2 => self.op_ld_a_ffc(),

      // 0xC3 => jp!(nn),
      0xD3 => self.op_nop(),
      0xE3 => self.op_nop(),
      0xF3 => self.op_di(),

      // 0xC4 => call!(nz),
      // 0xD4 => call!(nc),
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

      // 0xC7 => rst!(0x00),
      // 0xD7 => rst!(0x10),
      // 0xE7 => rst!(0x20),
      // 0xF7 => rst!(0x30),

      // 0xC8 => ret!(z),
      // 0xD8 => ret!(c),
      0xE8 => self.op_add_sp_dd(),
      0xF8 => self.op_ld_hl_sp_dd(),

      // 0xC9 => ret!(),
      // 0xD9 => reti!(),
      // 0xE9 => jp!(hl),
      0xF9 => self.op_ld_sp_hl(),

      // 0xCA => jp!(z),
      // 0xDA => jp!(c),
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

      // 0xCC => call!(z),
      // 0xDC => call!(c),
      0xEC => self.op_nop(),
      0xFC => self.op_nop(),

      // 0xCD => call!(nn),
      0xDD => self.op_nop(),
      0xED => self.op_nop(),
      0xFD => self.op_nop(),

      0xCE => self.op_adc_n(),
      0xDE => self.op_sbc_n(),
      0xEE => self.op_xor_n(),
      0xFE => self.op_cp_n(),

      // 0xCF => rst!(0x08),
      // 0xDF => rst!(0x18),
      // 0xEF => rst!(0x28),
      // 0xFF => rst!(0x38),

      _ => 0,
    }
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
