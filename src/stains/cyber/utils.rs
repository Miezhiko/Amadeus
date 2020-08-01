pub fn get_race(r : u32) -> String {
  String::from(
    match r {
      1 => "Human",
      2 => "Orc",
      4 => "Night Elf",
      8 => "Undead",
      _ => "Random"
    }
  )
}

pub fn get_race2(r : u32) -> String {
  String::from(
    match r {
      1 => "H",
      2 => "O",
      4 => "NE",
      8 => "UD",
      _ => "RND"
    }
  )
}

pub fn get_league(l: u32) -> String {
  String::from(match l {
    0 => "GrandMaster",
    1 => "Master",
    2 => "Diamond",
    3 => "Platinum",
    4 => "Gold",
    5 => "Silver",
    6 => "Bronze",
    _ => ""
  })
}

fn try_get_map(m: &str) -> String {
  String::from(match m {
    "Overall"               => "All",
    "echoisles"             => "EI",
    "northernisles"         => "NIS",
    "amazonia"              => "AZ",
    "lastrefuge"            => "LR",
    "concealedhill"         => "CH",
    "twistedmeadows"        => "TM",
    "terenasstand"          => "TS",
    "autumnleaves"          => "AL",
    "avalanche"             => "Avalanche",
    "goldshire"             => "Goldshire",
    "losttemple"            => "Lost Temple",
    "turtlerock"            => "Turtle Rock",
    "hillsbradcreek"        => "Hillsbrad Creek",
    "gnollwood"             => "Gnoll Wood",
    "tidewaterglades"       => "Tide Water Glades",
    "circleoffallenheroes"  => "Circle of Hate",
    _                       => ""})
}

pub fn get_map(m: &str) -> String {
  let mut map = try_get_map(m);
  if map.is_empty() {
    let split_ : Vec<String> = m.split('_')
                                .filter(|x| !x.is_empty())
                                .map(str::to_string)
                                .collect();
    if split_.len() == 3 { // alike _1v1_autumnleaves_anon
      map = try_get_map(split_[1].as_str())
    } else if split_.len() == 2 { // alike _gnollwood_anon
      map = try_get_map(split_[0].as_str())
    }
  }
  if map.is_empty() { m.to_string() } else { map }
}

pub fn get_league_png(lid: u32) -> String {
  String::from(match lid {
    0 => "https://www.w3champions.com/img/0.26f0662f.png",
    1 => "https://www.w3champions.com/img/1.9730fb2e.png",
    2 => "https://www.w3champions.com/img/2.48f016c5.png",
    3 => "https://www.w3champions.com/img/3.0fe1d052.png",
    4 => "https://www.w3champions.com/img/4.a255b682.png",
    5 => "https://www.w3champions.com/img/5.7f2f103c.png",
    6 => "https://www.w3champions.com/img/6.26efd96b.png",
    _ => ""
  })
}

#[cfg(test)]
mod base_dhall_tests {
  use super::*;
  #[test]
  fn get_map_test() {
    assert_eq!(get_map("_1v1_autumnleaves_anon"), "AL");
    assert_eq!(get_map("_gnollwood_anon"), "Gnoll Wood");
  }
}
