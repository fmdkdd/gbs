pub fn to_u16(h: u8, l: u8) -> u16 {
  ((h as u16) << 8) | (l as u16)
}

pub fn from_u16(hl: u16) -> (u8, u8) {
  ((hl >> 8) as u8, hl as u8)
}
