extern crate hound;
extern crate gbs;

use gbs::sound;

fn main() {
  // Emulate
  let mut sq1 = sound::Square1::new();
  let mut samples = Vec::new();

  sq1.set_frequency(440);
  sq1.set_length(500);
  samples.append(&mut sq1.get_samples_for(1000));
  sq1.set_frequency(880);
  sq1.set_length(500);
  samples.append(&mut sq1.get_samples_for(1000));

  // Write to WAV
  let spec = hound::WavSpec {
    channels: 1,
    sample_rate: gbs::sound::SAMPLE_RATE as u32,
    bits_per_sample: 16,
    sample_format: hound::SampleFormat::Int,
  };
  let max = std::i16::MAX as f32;
  let amp = 0.3;
  let mut writer = hound::WavWriter::create("out.wav", spec).unwrap();
  for s in samples {
    writer.write_sample((s * amp * max) as i16).unwrap();
  }
}
