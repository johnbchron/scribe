use std::{fs::File, io::BufReader, path::PathBuf};

use color_eyre::eyre::{Result, WrapErr};
use rodio::Source;

pub fn decode_audio_file_to_samples(path: PathBuf) -> Result<Vec<f32>> {
  let file =
    BufReader::new(File::open(path).wrap_err("failed to open audio file")?);
  let source = rodio::Decoder::new(file)
    .wrap_err("failed to decode audio")?
    .convert_samples();
  let num_channels = source.channels();
  let input_sample_rate = source.sample_rate();
  let samples_in = source.collect::<Vec<f32>>();
  let total_samples = samples_in.len();
  println!(
    "input file has {num_channels} channels at {input_sample_rate} hz, with \
     {total_samples} total samples"
  );
  println!(
    "max sample amplitude in input is {:?}",
    samples_in
      .iter()
      .map(|s| s.abs())
      .reduce(|v, a| f32::max(v, a))
      .unwrap()
  );

  let samples_out = samplerate::convert(
    input_sample_rate,
    16000,
    num_channels as _,
    samplerate::ConverterType::SincBestQuality,
    &samples_in,
  )
  .unwrap();

  Ok(samples_out)
}
