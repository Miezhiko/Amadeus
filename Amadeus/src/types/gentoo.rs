#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
pub struct Bug {
  pub assigned_to: String,
  pub creation_time: String,
  pub creator: String,
  pub is_open: bool,
  pub priority: String,
  pub product: String,
  pub severity: String,
  pub resolution: String,
  pub status: String,
  pub summary: String,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
pub struct Bugs {
  pub bugs: Vec<Bug>
}
