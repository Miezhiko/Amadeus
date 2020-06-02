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

pub fn get_map(m: &str) -> String {
  String::from(match m {
    "Overall"         => "All",
    "echoisles"       => "EI",
    "northernisles"   => "NIS",
    "amazonia"        => "AZ",
    "lastrefuge"      => "LR",
    "concealedhill"   => "CH",
    "twistedmeadows"  => "TM",
    "terenasstand"    => "TS",
    another_map       => another_map
  })
}
