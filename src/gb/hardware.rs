use gb::lcd::LCD;
use gb::apu::APU;

const RAM_SIZE : usize = 0x10000;

pub struct Hardware {
  pub ram: [u8; RAM_SIZE],
  lcd: LCD,
  apu: APU,
}

impl Hardware {
  pub fn new() -> Hardware {
    Hardware {
      ram: [0; RAM_SIZE],
      lcd: LCD::new(),
      apu: APU::new(),
    }
  }
}

impl Hardware {
  pub fn read(&self, addr: u16) -> u8 {
    match addr {
      0xE000...0xFDFF => self.read(addr - 0x2000),
      0xFF10...0xFF3F => self.apu.read(addr),
      0xFF40 => self.lcd.read(addr),
      0xFF42 => self.lcd.read(addr),
      0xFF43 => self.lcd.read(addr),
      0xFF44 => self.lcd.read(addr),
      0xFF47 => self.lcd.read(addr),
      _ => self.ram[addr as usize]
    }
  }

  pub fn write(&mut self, addr: u16, w: u8) {
    if cfg!(feature = "debug") {
      match addr {
        0xFF01 => println!("{:x} {}", w, ASCII[w as usize]),
        0xFF50 => println!("bingo"),
        _ => {},
      }
    }

    match addr {
      0xFF10...0xFF3F => self.apu.write(addr, w),
      0xFF40 => self.lcd.write(addr, w),
      0xFF42 => self.lcd.write(addr, w),
      0xFF43 => self.lcd.write(addr, w),
      0xFF44 => self.lcd.write(addr, w),
      0xFF47 => self.lcd.write(addr, w),
      _ => self.ram[addr as usize] = w
    }
  }

  pub fn apu_step(&mut self) {
    self.apu.step();
  }

  pub fn apu_output(&self) -> f32 {
    self.apu.output()
  }
}

// For running ROM tests
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
