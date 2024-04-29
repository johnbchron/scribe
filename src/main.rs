mod audio;

use std::{
  fmt::{self, Debug},
  path::PathBuf,
};

use clap::Parser;
use color_eyre::eyre::{Result, WrapErr};

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
  path: PathBuf,
}

struct TranscribedSegment {
  start: i64,
  end:   i64,
  text:  String,
}

impl Debug for TranscribedSegment {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "[{} - {}]: {:?}", self.start, self.end, self.text)
  }
}

#[derive(Debug)]
struct Transcription(Vec<TranscribedSegment>);

fn main() -> Result<()> {
  let args = Args::parse();

  let audio_file = args.path;
  let transcription = transcribe(audio_file)?;
  dbg!(transcription.0);

  Ok(())
}

fn transcribe(audio_file: PathBuf) -> Result<Transcription> {
  use whisper_rs::{
    FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters,
  };

  let samples = self::audio::decode_audio_file_to_samples(audio_file)
    .wrap_err("failed to decode audio file")?;
  println!("got {} audio samples", samples.len());

  // create the whisper context
  let ctx = WhisperContext::new_with_params(
    &std::env::var("MODEL_PATH")
      .wrap_err("env var MODEL_PATH not populated")?,
    WhisperContextParameters { use_gpu: true },
  )
  .wrap_err("failed to load whisper model")?;
  // create the model params
  let mut model_params =
    FullParams::new(SamplingStrategy::Greedy { best_of: 0 });
  model_params.set_print_special(false);
  model_params.set_print_progress(false);
  model_params.set_print_realtime(false);
  model_params.set_print_timestamps(false);

  // create the state
  let mut model_state = ctx
    .create_state()
    .wrap_err("failed to create model state")?;
  // start a full transcription
  model_state
    .full(model_params, &samples)
    .wrap_err("failed to run model")?;

  let num_segments: i32 = model_state
    .full_n_segments()
    .wrap_err("failed to get the number of segments")?;
  let mut segments: Vec<TranscribedSegment> =
    Vec::with_capacity(num_segments as _);
  println!("got {} segments", num_segments);

  for i in 0..num_segments {
    let segment_text = model_state
      .full_get_segment_text(i)
      .wrap_err("failed to get segment text")?;
    let start_timestamp = model_state
      .full_get_segment_t0(i)
      .wrap_err("failed to get segment start timestamp")?;
    let end_timestamp = model_state
      .full_get_segment_t1(i)
      .wrap_err("failed to get segment end timestamp")?;
    segments.push(TranscribedSegment {
      start: start_timestamp,
      end:   end_timestamp,
      text:  segment_text,
    });
  }

  Ok(Transcription(segments))
}
