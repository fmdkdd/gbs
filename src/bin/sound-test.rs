extern crate hound;
extern crate gbs;

use gbs::sound;

const SAMPLE_RATE: u32 = 44100;

fn main() {
  // Emulate
  let mut sq1 = sound::Square1::new();

  sq1.set_frequency(440);
  sq1.set_length(100);
  let samples = sq1.run(1000);

  // Write to WAV
  let spec = hound::WavSpec {
    channels: 1,
    sample_rate: SAMPLE_RATE,
    bits_per_sample: 8,
    sample_format: hound::SampleFormat::Int,
  };
  let max = 127f32;
  let amp = 0.3;
  let mut writer = hound::WavWriter::create("out.wav", spec).unwrap();
  for s in samples {
    writer.write_sample((s * amp * max) as i8).unwrap();
  }
}