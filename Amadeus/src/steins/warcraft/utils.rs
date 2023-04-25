use regex::Regex;
use once_cell::sync::Lazy;

static MAP_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(
r"^(?:s[0-9]+(?:[_|-][0-9]|))?(?:mapnames\.)?(?:_)?(?:[1-4]{1}v[1-4]{1}_)?([A-z._']+?)(?:w3c|w3x|roc)?(?:[0-9]+)?(?:v[0-9]+[_|-][0-9]+(?:[a-z])?)?(?:_lv|lv)?(?:_|\.)?(?:anon|w3m|nodraw)?(?:_|\.)?$"
).unwrap());

pub fn get_race(r: u32) -> String {
  String::from(
    match r { 1 => "H"
            , 2 => "O"
            , 4 => "NE"
            , 8 => "UD"
            , _ => "RND"})
}

pub fn get_league(l: u32) -> String {
  String::from(match l { 0 => "GrandMaster"
                       , 1 => "Master"
                       , 2 => "Adept"
                       , 3 => "Diamond"
                       , 4 => "Platinum"
                       , 5 => "Gold"
                       , 6 => "Silver"
                       , 7 => "Bronze"
                       , 8 => "Grass"
                       , _ => "" })
}

fn try_get_map_short(m: &str) -> String {
  String::from(
    match m { "overall"               => "All"
            , "echoisles"             => "EI"
            , "northernisles"         => "NIS"
            , "amazonia"              => "AZ"
            , "lastrefuge"            => "LR"
            , "concealedhill"         => "CH"
            , "twistedmeadows"        => "TM"
            , "terenasstand"          => "TS"
            , "autumnleaves"          => "AL"
            , "avalanche"             => "AV"
            , "losttemple"            => "LT"
            , "turtlerock"            => "TR"
            , "ruinsofazshara"        => "RoA"
            , "synergy"               => "SY"
            , "gnollwood"             => "GW"
            , "hillsbradcreek"        => "HC"
            , "goldshire"             => "GS"
            , "tidewaterglades"       => "TWG"
            , "circleoffallenheroes"  => "CoH"
            , "phantomgrove"          => "PG"
            , "fullscaleassault"      => "Ass"
            , "northshire"            => "NS"
            , "golemsinthemist"       => "GitM"
            , "wellspringtemple"      => "WS"
            , "marketsquare"          => "MS"
            , "deadlock"              => "DL"
            , "northernfelwood"       => "NFW"
            , "dragonfalls"           => "DF"
            , "ferocity"              => "Fro"
            , "murguloasis"           => "MO"
            , "twilightruins"         => "TR"
            , "goldrush"              => "GR"
            , "dalarangarden"         => "DG"
            , "nerubianpassage"       => "NM"
            , "cherryville"           => "CV"
            , "feralas"               => "Fera"
            , "battleground"          => "BG"
            , "sanctuary"             => "Sanc"
            , "fortpearl"             => "FP"
            , "tidehunters"           => "TH"
            , "royalgardens"          => "RG"
            , "shatteredexile"        => "SE"
            , "kal'drassil"           => "KD"
            , "shallowgrave"          => "SG"
            , "plunderisle"           => "PI"
            , "kingandcountry"        => "KaC"
            , "banditsretreat"        => "BB"
            , "northmarshruin"        => "NR"
            , "mur'guloasis"          => "MO"
            , "springtime"            => "ST"
            , "rustycreek"            => "RC"
            , "secretvalley"          => "SV"
            , "eversong"              => "ES"
            , _                       => "" })
}

fn try_get_map(m: &str) -> String {
  String::from(
    match m { "overall"               => "All"
            , "echoisles"             => "Echo Isles"
            , "northernisles"         => "Northern Isles"
            , "amazonia"              => "Amazonia"
            , "lastrefuge"            => "Last Refuge"
            , "concealedhill"         => "Concealed Hill"
            , "twistedmeadows"        => "Twisted Meadows"
            , "terenasstand"          => "Terenas Stand"
            , "autumnleaves"          => "Autumn Leaves"
            , "avalanche"             => "Avalanche"
            , "losttemple"            => "Lost Temple"
            , "turtlerock"            => "Turtle Rock"
            , "ruinsofazshara"        => "Ruins of Azshara"
            , "synergy"               => "Synergy"
            , "gnollwood"             => "Gnoll Wood"
            , "hillsbradcreek"        => "Hillsbrad Creek"
            , "goldshire"             => "Goldshire"
            , "tidewaterglades"       => "Tide Water Glades"
            , "circleoffallenheroes"  => "Circle of Hate"
            , "phantomgrove"          => "Phantom Grove"
            , "fullscaleassault"      => "Assault"
            , "northshire"            => "North Shire"
            , "golemsinthemist"       => "Gay Golems"
            , "wellspringtemple"      => "Well Spring Temple"
            , "marketsquare"          => "Базарная площадь"
            , "deadlock"              => "Deadlock"
            , "northernfelwood"       => "Northen Gay Wood"
            , "dragonfalls"           => "Dragon Balls"
            , "ferocity"              => "Ferocity"
            , "murguloasis"           => "Murgul Oasis"
            , "twilightruins"         => "Twilight Ruins"
            , "goldrush"              => "Gold Rush"
            , "dalarangarden"         => "Dalaran Garden"
            , "nerubianpassage"       => "Nerubian Massage"
            , "cherryville"           => "Cherry Ville"
            , "feralas"               => "Feralass"
            , "battleground"          => "Battleground"
            , "sanctuary"             => "Sanctuary"
            , "fortpearl"             => "Fort Pearl"
            , "tidehunters"           => "Tidehunters"
            , "royalgardens"          => "Royal Gardens"
            , "shatteredexile"        => "Shattered Exile"
            , "kal'drassil"           => "Kal'drassil"
            , "kaldrassil"            => "Kal'drassil"
            , "shallowgrave"          => "Shallow Grave"
            , "plunderisle"           => "Plunder Isle"
            , "kingandcountry"        => "King and Country"
            , "banditsretreat"        => "Bandits Betreat"
            , "northmarshruin"        => "Northmarsh Ruin"
            , "mur'guloasis"          => "Mur'gul Oasis"
            , "springtime"            => "Spring Time"
            , "rustycreek"            => "Rusty Creek"
            , "arathor"               => "Abathur"
            , "friends"               => "Gay Party"
            , "nightopia"             => "Niggopia"
            , "ekrezemsmaze"          => "Eklerzems Maze"
            , "paintedworld"          => "Painted World"
            , "snowblind"             => "Snow Blind"
            , "dejavu"                => "Dejavu"
            , "secretvalley"          => "Secret Valley"
            , "monopoly"              => "Monopoly"
            , "vanguardpoint"         => "Vanguard Point"
            , "chimneyisles"          => "Chimney Isles"
            , "whiteshore"            => "Black Shore"
            , _                       => m /*""*/ })
}

pub fn get_map(m: &str) -> String {
  let mut map = String::new();
  let lower_map = m.to_lowercase();
  if let Some(caps) = MAP_REGEX.captures(&lower_map) {
    if let Some(group1) = caps.get(1) {
      map = try_get_map(group1.as_str());
    }
  }
  if map.is_empty() { m.to_string() } else { map }
}

pub fn get_map_short(m: &str) -> String {
  let mut map = String::new();
  let lower_map = m.to_lowercase();
  if let Some(caps) = MAP_REGEX.captures(&lower_map) {
    if let Some(group1) = caps.get(1) {
      map = try_get_map_short(group1.as_str());
    }
  }
  if map.is_empty() { m.to_string() } else { map }
}

pub fn get_league_png(lid: u32) -> String {
  let base = "https://github.com/w3champions/w3champions-ui/raw/master/src/assets/leagueFlags/";
  format!("{base}{}.png"
    , match lid { 0 => "grandmaster"
                , 1 => "master"
                , 2 => "diamond"
                , 3 => "platinum"
                , 4 => "gold"
                , 5 => "silver"
                , 6 => "bronze"
                , _ => "unranked" })
}

pub fn get_hero_png(hero: &str) -> String {
  let base = "https://github.com/w3champions/w3champions-ui/raw/master/src/assets/heroes/";
  format!("{base}{hero}.png")
}

#[cfg(test)]
mod cyber_utils_tests {
  use super::*;
  #[test]
  fn get_map_test_basic() {
    assert_eq!(get_map("echoisles"), "Echo Isles");
    assert_eq!(get_map_short("echoisles"), "EI");
  }

  #[test]
  fn get_map_test_advanced() {
    assert_eq!(get_map_short("s12_1WellspringTemplev1_2"), "WS");
    assert_eq!(get_map_short("SecretValleyv2_0a"), "SV");

    assert_eq!(get_map("autumnleaves201016"), "Autumn Leaves");
    assert_eq!(get_map_short("autumnleaves201016"), "AL");

    assert_eq!(get_map("_1v1_autumnleaves_anon"), "Autumn Leaves");
    assert_eq!(get_map_short("_1v1_autumnleaves_anon"), "AL");

    assert_eq!(get_map("_gnollwood_anon"), "Gnoll Wood");
    assert_eq!(get_map_short("_gnollwood_anon"), "GW");

    assert_eq!(get_map_short("Springtimev1_1"), "ST");
    assert_eq!(get_map_short("s13SecretValleyv2_0"), "SV");

    assert_eq!(get_map("phantomgrovew3c201016"), "Phantom Grove");
    assert_eq!(get_map("PhantomGroveW3C"), "Phantom Grove");
    assert_eq!(get_map("DeadlockLV"), "Deadlock");
    assert_eq!(get_map("ShatteredExilev2_06"), "Shattered Exile");
    assert_eq!(get_map("BanditsRetreatv1_1"), "Bandits Betreat");
    assert_eq!(get_map("mapNames.ShatteredExilev2-07"), "Shattered Exile");

    assert_eq!(get_map("s14WellspringTemplev1_2nodraw"), "Well Spring Temple");
  }
}
