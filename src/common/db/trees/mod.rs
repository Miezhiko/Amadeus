pub mod points;
pub mod messages;
pub mod roles;
pub mod emojis;

use cannyls::{ nvm::FileNvm
             , storage::Storage };

use std::path::Path;

use tokio::sync::Mutex;

use once_cell::sync::Lazy;

pub static LSUF: &str = "trees/tree.lusf";
pub static ZSUF: &str = "trees/ztree.lusf";
pub static RSUF: &str = "trees/rtree.lusf";
pub static MSUF: &str = "trees/mtree.lusf";

fn get_storage(tree: &str) -> Storage<FileNvm> {
  if !Path::new(tree).exists() {
    let f = FileNvm::create(tree, 666666666).unwrap();
    let storage: Storage<FileNvm> = Storage::create(f).unwrap();
    storage
  } else {
    let f = FileNvm::open(tree).unwrap();
    let storage: Storage<FileNvm> = Storage::open(f).unwrap();
    storage
  }
}

pub static STORAGE: Lazy<Mutex<Storage<FileNvm>>> =
  Lazy::new(|| Mutex::new(get_storage(LSUF)));
pub static ZTREE: Lazy<Mutex<Storage<FileNvm>>> =
  Lazy::new(|| Mutex::new(get_storage(ZSUF)));
pub static RTREE: Lazy<Mutex<Storage<FileNvm>>> =
  Lazy::new(|| Mutex::new(get_storage(RSUF)));
pub static MTREE: Lazy<Mutex<Storage<FileNvm>>> =
  Lazy::new(|| Mutex::new(get_storage(MSUF)));
