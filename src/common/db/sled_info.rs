use eyre::{ WrapErr, Result };

static SLED: &str = "trees/info.sled";

pub fn store(key: &str, value: &str) -> Result<()> {
  let sled = sled::open(SLED)?;
  sled.insert(key, value)?;
  Ok(())
}

pub fn read(key: &str) -> Result<String> {
  let sled = sled::open(SLED)?;
  match sled.get(key) {
    Ok(Some(value)) => String::from_utf8(value.to_vec()).wrap_err("Failed to parse utf8"),
    Ok(None) => Err(eyre!("value not found")),
    Err(e) => Err(eyre!("operational problem encountered: {}", e))
  }
}

pub fn list() -> Result<String> {
  let sled = sled::open(SLED)?;
  let mut result = vec![];
  for key in sled.iter().keys() {
    if let Ok(k) = key {
      if let Ok(kk) = String::from_utf8(k.to_vec())
                            .wrap_err("Failed to parse utf8") {
        result.push(kk);
      }
    }
  }
  Ok(result.join("\n"))
}

pub fn delete(key: &str) -> Result<()> {
  let sled = sled::open(SLED)?;
  sled.remove(key)?;
  Ok(())
}
