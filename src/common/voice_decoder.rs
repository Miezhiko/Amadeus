use opus::{Channels, Decoder as OpusDecoder};

pub struct Decoder {
  pub opus_decoder: OpusDecoder,
}

unsafe impl Send for Decoder {}
unsafe impl Sync for Decoder {}

impl Decoder {
  pub fn new() -> Decoder {
    Decoder {
      opus_decoder: OpusDecoder::new(16_000, Channels::Stereo)
        .expect("something went wrong while making Opus decoder"),
    }
  }
}

impl Default for Decoder {
  fn default() -> Self {
    Self::new()
  }
}
