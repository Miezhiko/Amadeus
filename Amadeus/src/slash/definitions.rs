use serenity::{
  prelude::*,
  builder::{ CreateCommand, CreateCommandOption },
  model::{ guild::PartialGuild
         , application::CommandOptionType
         }
};

pub async fn create_app_commands(ctx: &Context, guild: &PartialGuild) {
  if let Err(why) = guild.set_application_commands(ctx, vec![
    CreateCommand::new("help").description("Display Amadeus Help"),
    CreateCommand::new("wave").description("Wave a hand you know..."),
    CreateCommand::new("cry").description("Start to cry!"),
    CreateCommand::new("cringe").description("just cringe"),
    CreateCommand::new("ahegao").description("Make an ahegao face"),
    CreateCommand::new("clap").description("Start clapping"),
    CreateCommand::new("shrug").description("Shrug shoulders"),
    CreateCommand::new("lol").description("laugh out loud"),
    CreateCommand::new("angry").description("Angry feels"),
    CreateCommand::new("dance").description("Dance Dance Dance"),
    CreateCommand::new("confused").description("Shows your confusion"),
    CreateCommand::new("shock").description("If you are shocked"),
    CreateCommand::new("nervous").description("Feeling nervous"),
    CreateCommand::new("sad").description("Feeling sad"),
    CreateCommand::new("happy").description("Feeling happy"),
    CreateCommand::new("annoyed").description("Really annoyed"),
    CreateCommand::new("omg").description("Oh my gawd"),
    CreateCommand::new("smile").description("Do a smile"),
    CreateCommand::new("ew").description("When you don't like something really"),
    CreateCommand::new("awkward").description("Feeling awkward"),
    CreateCommand::new("oops").description("This is just oops emotion..."),
    CreateCommand::new("lazy").description("Feeling lazy"),
    CreateCommand::new("hungry").description("Feeling hungry"),
    CreateCommand::new("stressed").description("Feeling stressed"),
    CreateCommand::new("scared").description("Really scared"),
    CreateCommand::new("bored").description("Feeling bored"),
    CreateCommand::new("yes").description("Yes Yes Yes"),
    CreateCommand::new("no").description("No No No"),
    CreateCommand::new("bye").description("Bye Bye"),
    CreateCommand::new("sorry").description("I am so sorry"),
    CreateCommand::new("sleepy").description("Feeling sleepy zzz"),
    CreateCommand::new("wink").description("Close and open one eye quickly"),
    CreateCommand::new("facepalm").description("A palm of a hand is brought to a face as an expression of dismay"),
    CreateCommand::new("whatever").description("you don't care"),
    CreateCommand::new("pout").description("do weird thing with lips"),
    CreateCommand::new("smug").description("showing an excessive pride in oneself"),
    CreateCommand::new("smirk").description("smile in an irritatingly smug, conceited, or silly way"),
    CreateCommand::new("hug").description("Literally hug someone")
      .add_option(CreateCommandOption::new(CommandOptionType::String, "person", "Person to hug")
          .required(true)
      ),
    CreateCommand::new("pat").description("Literally pat someone")
      .add_option(CreateCommandOption::new(CommandOptionType::String, "person", "Person to pat")
          .required(true)
      ),
    CreateCommand::new("slap").description("Literally slap someone")
      .add_option(CreateCommandOption::new(CommandOptionType::String, "person", "Person to slap")
          .required(true)
      ),
    CreateCommand::new("gif").description("Do some specific animation")
      .add_option(CreateCommandOption::new(CommandOptionType::String, "animation", "Search for specific animation")
          .required(true)
      ),
    CreateCommand::new("translate").description("Translate Russian to English")
      .add_option(CreateCommandOption::new(CommandOptionType::String, "text", "What will be translated")
          .required(true)
      ),
    CreateCommand::new("перевод").description("Перевод с английского на Русский")
      .add_option(CreateCommandOption::new(CommandOptionType::String, "текст", "Текст для перевода")
          .required(true)
      ),
    CreateCommand::new("stats").description("Display W3C player statistics")
      .add_option(CreateCommandOption::new(CommandOptionType::String, "battletag", "Target player")
          .required(true)
      ),
    CreateCommand::new("борис").description( "...")
      .add_option(CreateCommandOption::new(CommandOptionType::String, "текст", "Текст для Бориса")
          .required(true)
      ),
    CreateCommand::new("uwu").description("Uwufy some text OwO")
      .add_option(CreateCommandOption::new(CommandOptionType::String, "text", "Some text...")
          .required(true)
      ),
    CreateCommand::new("феминизировать").description("Феминизировать предложение")
      .add_option(CreateCommandOption::new(CommandOptionType::String, "текст", "Текст для феминизации")
          .required(true)
      ),
    CreateCommand::new("time").description("Display current time")
      .add_option(CreateCommandOption::new(CommandOptionType::String, "timezone", "Optional timezone")
          .required(false)
      ),
    CreateCommand::new("время").description("Показать текущее время")
      .add_option(CreateCommandOption::new(CommandOptionType::String, "город", "Дополнительный часовой пояс")
          .required(false)
      ),
    CreateCommand::new("join").description("Join voice channel with you (you should be in voice channel)"),
    CreateCommand::new("leave").description("Leave voice channel"),
    CreateCommand::new("repeat").description("Play last song again"),
    CreateCommand::new("play").description("Play radio stream or youtube stuff")
      .add_option(CreateCommandOption::new(CommandOptionType::String, "url", "link for music to play")
          .required(true)
      )
    ]
  ).await {
    error!("Failed to register global application commands {why}");
  }
}
