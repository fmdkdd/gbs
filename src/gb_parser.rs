use std::io::{self, BufReader, Read, Seek, SeekFrom};
use std::fs::File;
use std::path::Path;
use std::result;

use gbs_parser::read_binary::ReadBinary;

macro_rules! fail {
  ($err:path) => (return Err($err));
}

pub type Result<T> = result::Result<T, io::Error>;

#[derive(Debug)]
pub struct Gb {
  pub title:            String,
  pub cgb:              u8,
  pub new_licensee:     u16,
  pub sgb:              u8,
  pub cartridge_type:   u8,
  pub rom_size:         u8,
  pub ram_size:         u8,
  pub country_code:     u8,
  pub old_licensee:     u8,
  pub complement_check: u8,
  pub checksum:         u16,
  pub rom:              Vec<u8>,
}

pub fn load<P: AsRef<Path>>(path: P) -> self::Result<Gb> {
  let f = try!(File::open(path));
  let mut file = BufReader::new(f);

  file.seek(SeekFrom::Start(0x134))?;

  let title = file.read_str(15)?;
  let cgb = file.read_u8()?;
  let new_licensee = file.read_u16_be()?;
  let sgb = file.read_u8()?;
  let cartridge_type = file.read_u8()?;
  let rom_size = file.read_u8()?;
  let ram_size = file.read_u8()?;
  let country_code = file.read_u8()?;
  let old_licensee = file.read_u8()?;
  let _ = file.read_u8()?;
  let complement_check = file.read_u8()?;
  let checksum = file.read_u16_be()?;

  file.seek(SeekFrom::Start(0))?;
  let rom : Vec<u8> = try!(file.bytes().collect());

  Ok(Gb {
    title: title,
    cgb: cgb,
    new_licensee: new_licensee,
    sgb: sgb,
    cartridge_type: cartridge_type,
    rom_size: rom_size,
    ram_size: ram_size,
    country_code: country_code,
    old_licensee: old_licensee,
    complement_check: complement_check,
    checksum: checksum,
    rom: rom,
  })
}
