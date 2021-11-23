use crate::checks::*;
use crate::commands::{
  meta::*, chat::*
, warcraft::*, w3c::*
, owner::*, admin::*
, tictactoe::*, images::*
, translation::*, info::*
, bets::*, music::*
};

#[cfg(feature = "flo")]
use crate::commands::host::*;

use serenity::framework::standard::macros::group;

#[group("Meta")]
#[description = "Basic commands"]
#[commands( info, version, embed, qrcode, urban, uptime, tic_tac_toe, changelog
          , join, leave, play, repeat , help_ru, time )]
pub struct Meta;

#[group("Chat")]
#[description = "Chat commands"]
#[commands( quote, boris, owo, score, top, give, correct
          , feminize, extreme_feminize )]
pub struct Chat;

#[group("Translation")]
#[description = "Translation commands"]
#[commands(perevod, translate, en2de, de2en, en2fr, fr2en)]
pub struct Translate;

#[group("Images")]
#[description = "Gifs posting"]
#[commands(cry, hug, pat, slap, cringe, wave, sex, ahegao, clap, shrug, gifsearch
  , lol, angry, dance, confused, shock, nervous, sad, happy, annoyed, omg, smile
  , ew, awkward, oops, lazy, hungry, stressed, scared, bored, yes, no, bye, sorry
  , sleepy, wink, facepalm, whatever, pout, smug, smirk)]
pub struct Images;

#[group("Warcraft")]
#[description = "Warcraft events"]
#[commands(yesterday, today, tomorrow, weekends)]
pub struct Warcraft;

#[group("W3C")]
#[description = "w3champions commands"]
#[commands(stats, ongoing, veto, vs, bet)]
pub struct Pad;

#[group("Database")]
#[description = "Information storage commands"]
#[commands(register, show, delete, list)]
pub struct Info;

#[group("Owner")]
#[help_available(false)]
#[owners_only]
#[checks(Admin)]
#[commands(say, set, clear_messages, upgrade
  , update_cache, clear_chain_cache, unban_all
  , twitch_token_update, register_role, list_message_roles)]
pub struct Owner;

#[group("Admin")]
#[checks(Admin)]
#[help_available(false)]
#[commands(mute, unmute, eix, eix_update)]
pub struct Admin;

#[cfg(feature = "flo")]
#[group("Flo")]
#[help_available(false)]
#[commands(flo_nodes, flo_bans, register_player, register_me, host_vs, host_vs_amadeus)]
pub struct Flo;
