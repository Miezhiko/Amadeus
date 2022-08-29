use serenity::{
  prelude::*,
  builder::{ CreateApplicationCommand, CreateApplicationCommandOption },
  model::{ guild::PartialGuild
         , application::command::CommandOptionType
         }
};

pub async fn create_app_commands(ctx: &Context, guild: &PartialGuild) {
  if let Err(why) = guild.set_application_commands(ctx, vec![
    CreateApplicationCommand::new("help").description("Display Amadeus Help"),
    CreateApplicationCommand::new("wave").description("Wave a hand you know..."),
    CreateApplicationCommand::new("cry").description("Start to cry!"),
    CreateApplicationCommand::new("cringe").description("just cringe"),
    CreateApplicationCommand::new("ahegao").description("Make an ahegao face"),
    CreateApplicationCommand::new("clap").description("Start clapping"),
    CreateApplicationCommand::new("shrug").description("Shrug shoulders"),
    CreateApplicationCommand::new("lol").description("laugh out loud"),
    CreateApplicationCommand::new("angry").description("Angry feels"),
    CreateApplicationCommand::new("dance").description("Dance Dance Dance"),
    CreateApplicationCommand::new("confused").description("Shows your confusion"),
    CreateApplicationCommand::new("shock").description("If you are shocked"),
    CreateApplicationCommand::new("nervous").description("Feeling nervous"),
    CreateApplicationCommand::new("sad").description("Feeling sad"),
    CreateApplicationCommand::new("happy").description("Feeling happy"),
    CreateApplicationCommand::new("annoyed").description("Really annoyed"),
    CreateApplicationCommand::new("omg").description("Oh my gawd"),
    CreateApplicationCommand::new("smile").description("Do a smile"),
    CreateApplicationCommand::new("ew").description("When you don't like something really"),
    CreateApplicationCommand::new("awkward").description("Feeling awkward"),
    CreateApplicationCommand::new("oops").description("This is just oops emotion..."),
    CreateApplicationCommand::new("lazy").description("Feeling lazy"),
    CreateApplicationCommand::new("hungry").description("Feeling hungry"),
    CreateApplicationCommand::new("stressed").description("Feeling stressed"),
    CreateApplicationCommand::new("scared").description("Really scared"),
    CreateApplicationCommand::new("bored").description("Feeling bored"),
    CreateApplicationCommand::new("yes").description("Yes Yes Yes"),
    CreateApplicationCommand::new("no").description("No No No"),
    CreateApplicationCommand::new("bye").description("Bye Bye"),
    CreateApplicationCommand::new("sorry").description("I am so sorry"),
    CreateApplicationCommand::new("sleepy").description("Feeling sleepy zzz"),
    CreateApplicationCommand::new("wink").description("Close and open one eye quickly"),
    CreateApplicationCommand::new("facepalm").description("A palm of a hand is brought to a face as an expression of dismay"),
    CreateApplicationCommand::new("whatever").description("you don't care"),
    CreateApplicationCommand::new("pout").description("do weird thing with lips"),
    CreateApplicationCommand::new("smug").description("showing an excessive pride in oneself"),
    CreateApplicationCommand::new("smirk").description("smile in an irritatingly smug, conceited, or silly way"),
    CreateApplicationCommand::new("hug").description("Literally hug someone")
      .add_option(CreateApplicationCommandOption::new(CommandOptionType::String, "person", "Person to hug")
          .required(true)
      ),
    CreateApplicationCommand::new("pat").description("Literally pat someone")
      .add_option(CreateApplicationCommandOption::new(CommandOptionType::String, "person", "Person to pat")
          .required(true)
      ),
    CreateApplicationCommand::new("slap").description("Literally slap someone")
      .add_option(CreateApplicationCommandOption::new(CommandOptionType::String, "person", "Person to slap")
          .required(true)
      ),
    CreateApplicationCommand::new("gif").description("Do some specific animation")
      .add_option(CreateApplicationCommandOption::new(CommandOptionType::String, "animation", "Search for specific animation")
          .required(true)
      ),
    CreateApplicationCommand::new("translate").description("Translate Russian to English")
      .add_option(CreateApplicationCommandOption::new(CommandOptionType::String, "text", "What will be translated")
          .required(true)
      ),
    CreateApplicationCommand::new("перевод").description("Перевод с английского на Русский")
      .add_option(CreateApplicationCommandOption::new(CommandOptionType::String, "текст", "Текст для перевода")
          .required(true)
      ),
    CreateApplicationCommand::new("stats").description("Display W3C player statistics")
      .add_option(CreateApplicationCommandOption::new(CommandOptionType::String, "battletag", "Target player")
          .required(true)
      ),
    CreateApplicationCommand::new("борис").description( "...")
      .add_option(CreateApplicationCommandOption::new(CommandOptionType::String, "текст", "Текст для Бориса")
          .required(true)
      ),
    CreateApplicationCommand::new("uwu").description("Uwufy some text OwO")
      .add_option(CreateApplicationCommandOption::new(CommandOptionType::String, "text", "Some text...")
          .required(true)
      ),
    CreateApplicationCommand::new("феминизировать").description("Феминизировать предложение")
      .add_option(CreateApplicationCommandOption::new(CommandOptionType::String, "текст", "Текст для феминизации")
          .required(true)
      ),
    CreateApplicationCommand::new("time").description("Display current time")
      .add_option(CreateApplicationCommandOption::new(CommandOptionType::String, "timezone", "Optional timezone")
          .required(false)
      ),
    CreateApplicationCommand::new("время").description("Показать текущее время")
      .add_option(CreateApplicationCommandOption::new(CommandOptionType::String, "город", "Дополнительный часовой пояс")
          .required(false)
      ),
    CreateApplicationCommand::new("join").description("Join voice channel with you (you should be in voice channel)"),
    CreateApplicationCommand::new("leave").description("Leave voice channel"),
    CreateApplicationCommand::new("repeat").description("Play last song again"),
    CreateApplicationCommand::new("play").description("Play radio stream or youtube stuff")
      .add_option(CreateApplicationCommandOption::new(CommandOptionType::String, "url", "link for music to play")
          .required(true)
      )
    ]
  ).await {
    error!("Failed to register global application commands {why}");
  }
}
