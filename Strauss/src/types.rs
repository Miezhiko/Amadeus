use bincode::{ Decode, Encode };

#[derive(Encode, Decode, PartialEq, Eq, Debug)]
pub struct ChatResponse {
  pub message: Option<u64>,
  pub channel: u64,
  pub response: String,
  pub russian: bool
}
