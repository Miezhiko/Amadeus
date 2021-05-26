use dasp_interpolate::linear::Linear;
use dasp_signal::{from_iter, interpolate::Converter, Signal};
use deepspeech::errors::DeepspeechError;
use deepspeech::Model;
use std::path::Path;

// The model has been trained on this specific
// sample rate.
pub const SAMPLE_RATE: u32 = 16_000;

pub async fn run_stt(input_data: Vec<i16>) -> Result<String, DeepspeechError> {
  // Run the speech to text algorithm
  tokio::task::spawn_blocking(move || {
    let model_dir_str = "ds";
    let dir_path = Path::new(model_dir_str);
    let mut graph_name: Box<Path> = dir_path.join("output_graph.pb").into_boxed_path();
    let mut scorer_name: Option<Box<Path>> = None;
    // search for model in model directory
    for file in dir_path
      .read_dir()
      .expect("Specified model dir is not a dir")
    {
      if let Ok(f) = file {
        let file_path = f.path();
        if file_path.is_file() {
          if let Some(ext) = file_path.extension() {
            if ext == "pb" || ext == "pbmm" {
              graph_name = file_path.into_boxed_path();
            } else if ext == "scorer" {
              scorer_name = Some(file_path.into_boxed_path());
            }
          }
        }
      }
    }

    let mut m = Model::load_from_files(&graph_name).unwrap();
    // enable external scorer if found in the model folder
    if let Some(scorer) = scorer_name {
      println!("Using external scorer `{}`", scorer.to_str().unwrap());
      m.enable_external_scorer(&scorer).unwrap();
    }

    let interpolator = Linear::new([0i16], [0]);
    let conv = Converter::from_hz_to_hz(
      from_iter(input_data.iter().map(|v| [*v]).collect::<Vec<_>>()),
      interpolator,
      48000_f64,
      SAMPLE_RATE as f64,
    );
    let audio_buf: Vec<_> = conv.until_exhausted().map(|v| v[0]).collect();
    debug!("passing audio buf to stt: {:?}", &audio_buf);

    // Run the speech to text algorithm
    m.speech_to_text(&audio_buf)
  })
  .await
  .expect("Failed to spawn blocking!")
}
