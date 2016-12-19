pub mod cpu;
pub mod lcd;
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

  pub fn load_rom(&mut self, rom: &Vec<u8>, offset: u16) {
    // TODO: maybe use an iterator here or a direct memcpy
    // Copy ROM into RAM banks 0 and 1, stopping at 0x7fff, or when ROM is
    // empty.
    let mut ra = offset;
    let ra_max = 0x8000;
    let mut ro = 0;
    let ro_max = rom.len();
    while ro < ro_max && ra < ra_max {
      self.cpu.write(ra, rom[ro]);
      ra += 1;
      ro += 1;
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

  pub fn tile_pattern_table(&self) -> &[u8] {
    &self.cpu.bus.ram[0x8000..0x9000]
  }
}
