#[macro_export]
macro_rules! relative {
  ($f:expr) => {
    if cfg!(test) {
      concat!("../", $f)
    } else {
      $f
    }
  }
}

#[macro_export]
macro_rules! dhall {
  ($f:expr) => {
    serde_dhall::from_file($f).parse().unwrap()
  }
}
