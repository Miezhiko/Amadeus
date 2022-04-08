use bincode::{ Decode, Encode };

#[derive(Encode, Decode, PartialEq, Debug)]
pub struct ChatResponse {
  pub message: Option<u64>,
  pub channel: u64,
  pub response: String
}
