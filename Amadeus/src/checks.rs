use serenity::{
  client::Context,
  framework::standard::{ Args, CommandOptions
                       , Reason, macros::check },
  model::channel::Message
};

#[check]
#[name = "Moderator"]
#[check_in_help(true)]
#[display_in_help(true)]
pub async fn moderator_check( ctx: &Context
                            , msg: &Message
                            , _: &mut Args
                            , _: &CommandOptions ) -> Result<(), Reason> {
  if let Ok(member) = msg.member(ctx).await {
    if let Ok(permissions) = member.permissions(ctx) {
      if permissions.ban_members() {
        return Ok(());
      }
    }
  }
  Err(Reason::User("Lacked moderator permission".to_string()))
}

#[check]
#[name = "Admin"]
#[check_in_help(true)]
#[display_in_help(true)]
pub async fn admin_check( ctx: &Context
                        , msg: &Message
                        , _: &mut Args
                        , _: &CommandOptions ) -> Result<(), Reason> {
  if let Ok(member) = msg.member(ctx).await {
    if let Ok(permissions) = member.permissions(ctx) {
      if permissions.administrator() {
        return Ok(());
      } else if let Ok(info) = ctx.http.get_current_application_info().await {
        if let Some(team) = info.team {
          if msg.author.id == team.owner_user_id {
            return Ok(());
          }
        } else if let Some(owner) = info.owner {
          if msg.author.id == owner.id {
            return Ok(());
          }
        } else if msg.author.id.get() == 510368731378089984 {
          return Ok(());
        }
      }
    }
  }
  Err(Reason::User("Lacked admin permission".to_string()))
}
