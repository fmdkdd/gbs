enum SweepDirection {
  DECREASE, INCREASE
}

enum ShiftRegisterWidth {
  W_15BITS, 7BITS
}

// TODO: most of these are not 8 bit numbers
struct Square1 {
  // Sweep register
  sweep_period: u8,
  sweep_direction: SweepDirection,
  sweep_shift: u8,

  wave_pattern_duty: u8,
  length_load: u8,

  initial_volume: u8,
  volume_sweep_direction: SweepDirection,
  sweep_length: u8,

  frequency: u16,
  shift_clock_freq: u8,
  shift_register_width: ShiftRegisterWidth,
  freq_dividing_ratio: u8,

  length_counter_enable: bool,
  hh




}
