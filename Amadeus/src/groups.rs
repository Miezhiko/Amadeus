use crate::checks::*;
use crate::commands::{
  meta::*, chat::*
, warcraft::*, w3c::*
, owner::*, moderator::*
, tictactoe::*, images::*
, info::*, music::*, gentoo::*
, translation::*
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
#[commands( quote, boris, owo, score, top, give
          , feminize, extreme_feminize )]
pub struct Chat;

#[group("Translation")]
#[description = "Translation commands"]
#[commands( perevod, translate, en2de, de2en
          , en2fr, fr2en, ua2ru, ru2ua )]
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
#[commands(stats, ongoing, veto, vs, regenerate_stats)]
pub struct Pad;

#[group("Database")]
#[description = "Information storage commands"]
#[commands(register, show, delete, list)]
pub struct Info;

#[group("Owner")]
#[help_available(false)]
#[owners_only]
#[checks(Admin)]
#[commands(say, set, clear_messages, upgrade, catch_up_with_roles
  , update_cache, clear_chain_cache, unban_all, eix, ban, restart_kalmarity
  , twitch_token_update, register_role, list_message_roles)]
pub struct Owner;

#[group("Moderator")]
#[checks(Moderator)]
#[commands( mute, unmute, move_discussion, timeout, untimeout
          , j, prison, purge, dice_giveaway )]
pub struct Moderator;

#[cfg(feature = "flo")]
#[group("Flo")]
#[help_available(false)]
#[commands(flo_nodes, flo_bans, register_player, register_me, host_vs, host_vs_amadeus)]
pub struct Flo;

#[group("Gentoo")]
#[help_available(true)]
#[commands(bug, zugaina, wiki)]
pub struct Gentoo;
