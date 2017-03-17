pub mod cpu;
pub mod lcd;
pub mod apu;
pub mod bus;

mod utils;

use self::cpu::Cpu;
use self::bus::Hardware;

pub struct GB {
  pub cpu: Cpu<Hardware>,
}

impl GB {
  pub fn new() -> GB {
    GB {
      cpu: Cpu::new(Hardware::new()),
    }
  }

  pub fn load_rom(&mut self, rom: &[u8], offset: u16) {
    // Copy ROM into RAM at offset, stopping at 0x7fff, or when ROM is empty.
    for (addr,idx) in (offset..0x8000).zip(0..rom.len()) {
      self.cpu.write(addr, rom[idx]);
    }
  }

  pub fn reset(&mut self) {
    self.cpu.reset();
  }

  pub fn run_for(&mut self, cycles: u64) {
    let mut c : u64 = 0;

    while c < cycles {
      c += self.cpu.step() as u64;
    }
  }

  pub fn run(&mut self) {
    loop {
      self.cpu.step();
    }
  }

  pub fn tile_map(&self) -> &[u8] {
    let select = (self.cpu.read(0xFF40) & 0x08) > 0;
    match select {
      false => &self.cpu.bus.ram[0x9800..0x9C00],
      true => &self.cpu.bus.ram[0x9C00..0xA000],
    }
  }

  pub fn tile_data(&self) -> &[u8] {
    let select = (self.cpu.read(0xFF40) & 0x10) > 0;
    match select {
      false => &self.cpu.bus.ram[0x8800..0x9800],
      true => &self.cpu.bus.ram[0x8000..0x9000],
    }
  }

  pub fn tile_pattern_table(&self) -> &[u8] {
    &self.cpu.bus.ram[0x8000..0x9000]
  }
}
