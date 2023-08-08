use anyhow::Result;

use once_cell::sync::OnceCell;

use tokio::sync::Mutex;

static SLED_DB: OnceCell<Mutex<sled::Db>> = OnceCell::new();
static SLED: &str = "trees/info.sled";

async fn init_db() -> Result<()> {
  if let Some(_existing_handle) = SLED_DB.get() {
    Ok(())
  } else {
    let sled = sled::open(SLED)?;
    SLED_DB.set(
      Mutex::new(sled)
    ).map_err(|_| anyhow!("Failed to store db handle"))?;
    Ok(())
  }
}

pub async fn store(key: &str, value: &str) -> Result<()> {
  init_db().await?;
  if let Some(sled_mutex) = SLED_DB.get() {
    sled_mutex.lock().await.insert(key, value)?;
  }
  Ok(())
}

pub async fn read(key: &str) -> Result<String> {
  init_db().await?;
  if let Some(sled_mutex) = SLED_DB.get() {
    let sled = sled_mutex.lock().await;
    match sled.get(key) {
      Ok(Some(value)) => {
        String::from_utf8(value.to_vec())
          .map_err(|error| anyhow!("Failed to parse utf8 {error}"))
      },
      Ok(None) => Err(anyhow!("value not found")),
      Err(e) => Err(anyhow!("operational problem encountered: {e}"))
    }
  } else {
    Err(anyhow!("can't get sled db mutex"))
  }
}

pub async fn list() -> Result<String> {
  init_db().await?;
  if let Some(sled_mutex) = SLED_DB.get() {
    let sled = sled_mutex.lock().await;
    let mut result = vec![];
    for k in sled.iter().keys().flatten() {
      if let Ok(kk) = String::from_utf8(k.to_vec())
                        .map_err(|error| anyhow!("Failed to parse utf8 {error}")) {
        result.push(kk);
      }
    }
    Ok(result.join("\n"))
  } else {
    Err(anyhow!("can't get sled db mutex"))
  }
}

pub async fn delete(key: &str) -> Result<()> {
  init_db().await?;
  if let Some(sled_mutex) = SLED_DB.get() {
    let sled = sled_mutex.lock().await;
    sled.remove(key)?;
    Ok(())
  } else {
    Err(anyhow!("can't get sled db mutex"))
  }
}
