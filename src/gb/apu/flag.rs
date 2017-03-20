// Using this enum instead of bools allows us to simply cast as u8 to get a
// numeric value back
#[derive(Debug,Copy,Clone)]
pub enum Flag {
  Off = 0,
  On = 1,
}

impl From<bool> for Flag {
  fn from(v: bool) -> Self {
    if v { Flag::On }
    else { Flag::Off }
  }
}

impl From<Flag> for bool {
  fn from(v: Flag) -> Self {
    match v {
      Flag::Off => false,
      Flag::On => true,
    }
  }
}
