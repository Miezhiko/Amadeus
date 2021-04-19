
use serenity::{
  client::Context,
  framework::standard::{ Args, CommandOptions
                       , Reason, macros::check },
  model::channel::Message
};

#[check]
#[name = "Admin"]
#[check_in_help(true)]
#[display_in_help(true)]
pub async fn admin_check( ctx: &Context
                        , msg: &Message
                        , _: &mut Args
                        , _: &CommandOptions ) -> Result<(), Reason> {
  if let Ok(member) = msg.member(ctx).await {
    if let Ok(permissions) = member.permissions(&ctx).await {
      if permissions.administrator() {
        return Ok(());
      }
    }
  }
  Err(Reason::User("Lacked admin permission".to_string()))
}
