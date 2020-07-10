use sled;

use {
  byteorder::{BigEndian},
  zerocopy::{
    byteorder::U64, AsBytes, FromBytes, LayoutVerified, Unaligned
  }
};


#[derive(FromBytes, AsBytes, Unaligned)]
#[repr(C)]
struct Key {
  guild: U64<BigEndian>,
  user: U64<BigEndian>
}

#[derive(FromBytes, AsBytes, Unaligned)]
#[repr(C)]
struct Points {
  count: U64<BigEndian>,
  role: U64<BigEndian>
}

pub async fn add_points(guild_id: &u64, user_id: &u64, points: &u64) {
  let tree = sled::open("tree").expect("Open tree");
  let key = Key { guild: U64::new(*guild_id), user: U64::new(*user_id) };
  tree.update_and_fetch(key.as_bytes(), |value_opt| {
    if let Some(existing) = value_opt {
      let mut backing_bytes = sled::IVec::from(existing);
      let layout: LayoutVerified<&mut [u8], Points> =
        LayoutVerified::new_unaligned(&mut *backing_bytes)
          .expect("bytes do not fit schema");
      let value: &mut Points = layout.into_mut();
      let new_points = value.count.get() + points;
      value.count.set(new_points);
      Some(backing_bytes)
    } else {
      Some(sled::IVec::from(
        Points { count: U64::new(1), role: U64::new(0) }.as_bytes(),
      ))
    }
  }).expect("Update points");
  tree.flush_async().await.expect("Tree flush");
}

pub fn get_points(guild_id: &u64, user_id: &u64) -> u64 {
  let tree = sled::open("tree").expect("Open tree");
  let key = Key { guild: U64::new(*guild_id), user: U64::new(*user_id) };
  if let Ok(existing) = tree.get(key.as_bytes()) {
    if let Some(ivec) = existing {
      let layout: LayoutVerified<&[u8], Points> =
        LayoutVerified::new_unaligned(&*ivec)
          .expect("bytes do not fit schema");
      let value: &Points = layout.into_ref();
      value.count.get()
    } else {
      0
    }
  } else {
    0
  }
}
