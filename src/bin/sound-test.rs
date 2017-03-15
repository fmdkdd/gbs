extern crate hound;
extern crate gbs;

use gbs::sound;

fn main() {
  // Emulate
  let mut sq1 = sound::Square1::new();
  let mut samples = Vec::new();

  println!("First note");
  sq1.set_frequency(440.0);
  sq1.set_length(0.5);
  sq1.set_volume_init(1.0, 1.0);
  samples.append(&mut sq1.get_samples_for(1.0));
  println!("Second note");
  sq1.set_frequency(880.0);
  sq1.set_length(0.5);
  sq1.set_volume_init(1.0, 2.0);
  samples.append(&mut sq1.get_samples_for(1.0));

  println!("Vibrato");
  sq1.set_length(1.0);
  sq1.set_frequency(440.0);
  sq1.set_volume_init(1.0, 10.0);
  for i in 0..5u32 {
    sq1.set_frequency(440.0 + 1.0);
    samples.append(&mut sq1.get_samples_for(0.1));
    sq1.set_frequency(440.0 - 1.0);
    samples.append(&mut sq1.get_samples_for(0.1));
  }


  // Write to WAV
  let spec = hound::WavSpec {
    channels: 1,
    sample_rate: gbs::sound::SAMPLE_RATE as u32,
    bits_per_sample: 16,
    sample_format: hound::SampleFormat::Int,
  };
  let max = std::i16::MAX as f64;
  let amp = 0.3;
  let mut writer = hound::WavWriter::create("out.wav", spec).unwrap();
  for s in samples {
    writer.write_sample((s * amp * max) as i16).unwrap();
  }
}
