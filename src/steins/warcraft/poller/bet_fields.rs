use crate::types::tracking::TrackingGame;

use serenity::{ prelude::*
              , model::id::UserId };

pub async fn generate_bet_fields( ctx: &Context
                                , track: &mut TrackingGame
                                ) -> Option<Vec<(String, String, bool)>> {
  let mut bet_fields = None;
  if !track.bets.is_empty() {
    setm!{ woutput = vec![]
         , loutput = vec![] };
    for bet in &mut track.bets {
      let user_id = UserId( bet.member );
      if let Ok(user) = user_id.to_user(ctx).await {
        if bet.positive {
          woutput.push(
            format!("**{}**: {}", user.name, bet.points)
          );
        } else {
          loutput.push(
            format!("**{}**: {}", user.name, bet.points)
          );
        }
        bet.registered = true;
      }
    }
    let mut fstring = woutput.join("\n");
    if !loutput.is_empty() {
      let need_space = if woutput.is_empty() { "" } else { "\n" };
      fstring = format!("{}{}*on lose:*\n{}", fstring
                                            , need_space
                                            , loutput.join("\n"));
    }
    bet_fields = Some(vec![("Bets".to_string()
                          , fstring
                          , false)]);
  }
  bet_fields
}
