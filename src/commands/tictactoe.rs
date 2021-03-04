/*
 * Original author nitsuga5124
 * Licensed under Mozilla Public License Version 2.0
 * Whole license text: https://choosealicense.com/licenses/mpl-2.0
 */

use crate::{
  common::{ db::trees
          , msg::channel_message }
};

use std::fmt::Display;
use std::time::Duration;

use serenity::{
  prelude::Context,
  model::misc::Mentionable,
  model::guild::Member,
  model::channel::{ Message
                  , ReactionType },
  model::id::UserId,
  framework::standard::{ Args
                       , CommandResult
                       , macros::command }
};

use regex::Regex;
use once_cell::sync::Lazy;
use futures_util::{
  stream,
  StreamExt,
};

pub async fn parse_member(ctx: &Context, msg: &Message, member_name: String) -> Result<Member, String> {
  let mut members = Vec::new();
  if let Ok(id) = member_name.parse::<u64>() {
    let member = &msg.guild_id.unwrap().member(ctx, id).await;
    match member {
      Ok(m) => Ok(m.to_owned()),
      Err(why) => Err(why.to_string()),
    }
  } else if member_name.starts_with("<@") && member_name.ends_with('>') {
    static RE: Lazy<Regex> =
      Lazy::new(|| Regex::new("[<@!>]").unwrap());
    let member_id = RE.replace_all(&member_name, "").into_owned();
    let member = &msg.guild_id.unwrap().member(ctx, UserId(member_id.parse::<u64>().unwrap())).await;
    match member {
      Ok(m) => Ok(m.to_owned()),
      Err(why) => Err(why.to_string()),
    }
  } else {
    let guild = &msg.guild(ctx).await.unwrap();
    let member_name = member_name.split('#').next().unwrap();
    for m in guild.members.values() {
      if m.display_name() == std::borrow::Cow::Borrowed(member_name) ||
        m.user.name == member_name
      {
        members.push(m);
      }
    }
    if members.is_empty() {
      let similar_members = &guild.members_containing(&member_name, false, false).await;
      let mut members_string =  stream::iter(similar_members.iter())
        .map(|m| async move {
          let member = &m.0.user;
          format!("`{}`|", member.name)
        })
        .fold(String::new(), |mut acc, c| async move {
          acc.push_str(&c.await);
          acc
        }).await;
      let message = {
        if members_string.is_empty() {
          format!("No member named '{}' was found.", member_name.replace("@", ""))
        } else {
          members_string.pop();
          format!("No member named '{}' was found.\nDid you mean: {}", member_name.replace("@", ""), members_string.replace("@", ""))
        }
      };
      Err(message)
    } else if members.len() == 1 {
      Ok(members[0].to_owned())
    } else {
      let mut members_string =  stream::iter(members.iter())
        .map(|m| async move {
          let member = &m.user;
          format!("`{}#{}`|", member.name, member.discriminator)
        })
        .fold(String::new(), |mut acc, c| async move {
          acc.push_str(&c.await);
          acc
        }).await;
      members_string.pop();
      let message = format!("Multiple members with the same name where found: '{}'", &members_string);
      Err(message)
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Pieces {
  Cross,
  Circle,
}

#[derive(Debug, Clone, Copy)]
struct Player(UserId, Pieces);

#[derive(Default, Debug)]
struct Piece {
  pos_x: usize,
  pos_y: usize,
  typ: Option<Pieces>,
}

#[derive(Default, Debug)]
struct Board {
  table: [Piece; 9],
  current_piece: Pieces,
  win_condition: Option<Pieces>,
}

impl Default for Pieces {
  fn default() -> Self { Pieces::Cross }
}

impl Display for Pieces {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", match self {
      Self::Cross => "X",
      Self::Circle => "O",
    })
  }
}

impl Display for Board {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let mut board = format!("{} | A | B | C\n", self.current_piece);
    board += "--------------";
    let mut x = 0;
    for (index, i) in self.table.iter().enumerate() {
      if index % 3 == 0 {
        x+=1;
        board += &format!("\n{} ", x);
      }
      board += &format!("| {} ", {
        if let Some(piece) = &i.typ {
          piece.to_string()
        } else {
          " ".to_string()
        }
      });
    }
    write!(f, "{}", board)
  }
}

impl Board {
  fn place_piece(&mut self, piece: Piece) -> Result<(), ()> {
    let x = piece.pos_x * 3;
    let y = piece.pos_y % 3;
    if self.table[x+y].typ.is_none() {
      self.table[x+y] = piece;
      Ok(())
    } else {
      Err(())
    }
  }
  fn swap_current_piece(&mut self) {
    self.current_piece = match self.current_piece {
      Pieces::Cross => Pieces::Circle,
      Pieces::Circle => Pieces::Cross,
    }
  }
  fn check_win_condition(&mut self) {
    let win_conditions = [[0,1,2], [3,4,5], [6,7,8], [0,3,6], [1,4,7], [2,5,8], [0,4,8], [6,4,2]];
    for i in &win_conditions {
      if self.table[i[0]].typ == Some(Pieces::Cross) &&
         self.table[i[1]].typ == Some(Pieces::Cross) &&
         self.table[i[2]].typ == Some(Pieces::Cross)
      {
        self.win_condition = Some(Pieces::Cross);
      }
      if self.table[i[0]].typ == Some(Pieces::Circle) &&
         self.table[i[1]].typ == Some(Pieces::Circle) &&
         self.table[i[2]].typ == Some(Pieces::Circle)
      {
        self.win_condition = Some(Pieces::Circle);
      }
    }
  }
}

#[command]
#[aliases(ttt, tictactoe, крестики_нолики)]
#[min_args(1)]
#[description("play tictactoe (optionally for points)")]
async fn tic_tac_toe(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
  let other_player_copy = parse_member(ctx, &msg, args.single_quoted::<String>()?).await?;
  let other_player = &other_player_copy.user;
  let points_count =
    if let Ok(first) = args.single::<u64>() { first
    } else if let Ok(second) = args.advance().single::<u64>() { second
    } else { 0
    };

  let guild_id = msg.guild_id.unwrap();
  if points_count > 0 {
    if let Ok(p1) = trees::get_points( guild_id.0, msg.author.id.0 ).await {
      if p1 < points_count {
        let err = format!("{} only has {}, need {}", msg.author.name, p1, points_count);
        channel_message(ctx, msg, &err).await;
        return Ok(());
      }
    }
    if let Ok(p2) = trees::get_points( guild_id.0, other_player.id.0 ).await {
      if p2 < points_count {
        let err = format!("{} only has {}, need {}", other_player.name, p2, points_count);
        channel_message(ctx, msg, &err).await;
        return Ok(());
      }
    }
  }
  let mut confirmation = msg.channel_id.say(ctx, format!("{}: Will you play TicTacToe for {} points?", other_player.mention(), points_count)).await?;
  confirmation.react(ctx, '✅').await?;
  confirmation.react(ctx, '❌').await?;
  loop {
    if let Some(reaction) = other_player.await_reaction(ctx).timeout(Duration::from_secs(120)).await {
      let emoji = &reaction.as_inner_ref().emoji;
      match emoji.as_data().as_str() {
        "✅" => {
          confirmation.delete(ctx).await?;
          break;
        },
        "❌" => {
          confirmation.edit(ctx, |m| m.content(
            format!(
              "{}: {} didn't accept the match.",
              msg.author.mention(), other_player.mention()
            )
          )).await?;
          return Ok(());
        },
        _ => ()
      }
    } else {
      confirmation.edit(ctx, |m| m.content(
        format!(
          "{}: {} took to long to respond.",
          msg.author.mention(), other_player.mention()
        )
      )).await?;
      return Ok(());
    }
  }
  let mut players = [
    Player(msg.author.id, Pieces::Cross),
    Player(other_player.id, Pieces::Circle),
  ].repeat(5);
  if msg.timestamp.timestamp() % 2 == 0 {
    players.reverse();
  }
  players.pop();
  let mut board = Board {
    current_piece: players[0].1,
    ..Default::default()
  };
  board.current_piece = players[0].1;
  let mut m = msg.channel_id.say(ctx, format!(">>> ```{}```", &board)).await?;
  for i in 1..4u8 {
    let num = ReactionType::Unicode(format!("{}\u{fe0f}\u{20e3}", i));
    m.react(ctx, num).await?;
  }
  let _a = ReactionType::Unicode(String::from("\u{01f1e6}"));
  let _b = ReactionType::Unicode(String::from("\u{01f1e7}"));
  let _c = ReactionType::Unicode(String::from("\u{01f1e8}"));
  m.react(ctx, _a).await?;
  m.react(ctx, _b).await?;
  m.react(ctx, _c).await?;
  for i in &players {
    m.edit(ctx, |m| m.content(format!("{}\n>>> ```{}```", i.0.mention(), &board))).await?;
    'outer: loop {
      let mut x: Option<usize> = None;
      let mut y: Option<usize> = None;
      loop {
        if x.is_none() || y.is_none() {
          if let Some(reaction) = i.0.to_user(ctx).await?.await_reaction(ctx).timeout(Duration::from_secs(120)).await {
            let _ = reaction.as_inner_ref().delete(ctx).await;
            let emoji = &reaction.as_inner_ref().emoji;

            match emoji.as_data().as_str() {
              "1\u{fe0f}\u{20e3}" => x = Some(0),
              "2\u{fe0f}\u{20e3}" => x = Some(1),
              "3\u{fe0f}\u{20e3}" => x = Some(2),
              "\u{01f1e6}" => y = Some(0),
              "\u{01f1e7}" => y = Some(1),
              "\u{01f1e8}" => y = Some(2),
              _ => ()
            }
          } else {
            m.edit(ctx, |m| m.content(format!("{}: Timeout", i.0.mention()))).await?;
            let _ = m.delete_reactions(ctx).await;
            return Ok(());
          }
        } else if !x.is_none() && !y.is_none() {
          let piece = Piece {
            pos_x: x.unwrap_or(0),
            pos_y: y.unwrap_or(0),
            typ: Some(i.1),
          };
          if board.place_piece(piece).is_err() {
            x = None;
            y = None;
          } else {
            break 'outer
          }
        }
      }
    }
    board.check_win_condition();
    if board.win_condition.is_some() {
      m.edit(ctx, |m| m.content(format!("{} WON!\n>>> ```{}```", i.0.mention(), &board))).await?;
      let _ = m.delete_reactions(ctx).await;
      if points_count > 0 {
        let (loser, winner) =
          if msg.author.id == i.0 {
            (other_player, &msg.author)
          } else {
            (&msg.author, other_player)
          };
        let (succ, rst) = trees::give_points( guild_id.0
                                            , loser.id.0
                                            , winner.id.0
                                            , points_count ).await;
        if succ {
          let out = format!("{} to {}", rst, winner.name);
          if let Err(why) = msg.channel_id.send_message(ctx, |m| m
            .embed(|e| e
            .description(&out)
            .footer(|f| f.text(&loser.name))
          )).await {
            error!("Failed to post give {:?}", why);
          }
        } else {
          channel_message(ctx, msg, &rst).await;
        }
      }
      return Ok(());
    }
    board.swap_current_piece();
  }
  m.edit(ctx, |m| m.content(format!("{} and {} tied.\n>>> ```{}```", players[0].0.mention(), players[1].0.mention(), &board))).await?;
  let _ = m.delete_reactions(ctx).await;
  Ok(())
}
