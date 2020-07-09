use sled;

use zerocopy::{
  AsBytes, FromBytes, LayoutVerified
};

#[derive(FromBytes, AsBytes)]
#[repr(C)]
struct Points {
  count: u64,
  role: u64
}

//TODO: RECODE ALL

pub fn add_points(user_id: &u64, points: &u64) {
  let tree = sled::open("tree").expect("Open tree");
  let key = user_id.to_be_bytes();
  tree.update_and_fetch(&key, |value_opt| {
    if let Some(existing) = value_opt {
      let mut backing_bytes = sled::IVec::from(existing);
      let layout: LayoutVerified<&mut [u8], Points> =
        LayoutVerified::new(&mut *backing_bytes)
            .expect("bytes do not fit schema");
      let value: &mut Points = layout.into_mut();
      value.count += points;
      Some(backing_bytes)
    } else {
      Some(sled::IVec::from(
        Points { count: 1, role: 0 }.as_bytes(),
      ))
    }
  }).expect("Update points");
  tree.flush().expect("Tree flush");
}

pub fn get_points(user_id: &u64) -> u64 {
  let tree = sled::open("tree").expect("Open tree");
  let key = user_id.to_be_bytes();
  if let Ok(existing) = tree.get(key) {
    if let Some(ivec) = existing {
      let layout: LayoutVerified<&[u8], Points> =
        LayoutVerified::new(&*ivec).expect("bytes do not fit schema");
      let value: &Points = layout.into_ref();
      value.count
    } else {
      0
    }
  } else {
    0
  }
}
