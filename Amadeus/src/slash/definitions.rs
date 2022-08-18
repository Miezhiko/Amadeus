use serenity::{
  prelude::*,
  builder::{ CreateApplicationCommand, CreateApplicationCommandOption },
  model::{ guild::PartialGuild
         , application::command::CommandOptionType
         }
};

pub async fn create_app_commands(ctx: &Context, guild: &PartialGuild) {
  if let Err(why) = guild.set_application_commands(ctx, vec![
    CreateApplicationCommand::new("help",     "Display Amadeus Help"),
    CreateApplicationCommand::new("wave",     "Wave a hand you know..."),
    CreateApplicationCommand::new("cry",      "Start to cry!"),
    CreateApplicationCommand::new("cringe",   "just cringe"),
    CreateApplicationCommand::new("ahegao",   "Make an ahegao face"),
    CreateApplicationCommand::new("clap",     "Start clapping"),
    CreateApplicationCommand::new("shrug",    "Shrug shoulders"),
    CreateApplicationCommand::new("lol",      "laugh out loud"),
    CreateApplicationCommand::new("angry",    "Angry feels"),
    CreateApplicationCommand::new("dance",    "Dance Dance Dance"),
    CreateApplicationCommand::new("confused", "Shows your confusion"),
    CreateApplicationCommand::new("shock",    "If you are shocked"),
    CreateApplicationCommand::new("nervous",  "Feeling nervous"),
    CreateApplicationCommand::new("sad",      "Feeling sad"),
    CreateApplicationCommand::new("happy",    "Feeling happy"),
    CreateApplicationCommand::new("annoyed",  "Really annoyed"),
    CreateApplicationCommand::new("omg",      "Oh my gawd"),
    CreateApplicationCommand::new("smile",    "Do a smile"),
    CreateApplicationCommand::new("ew",       "When you don't like something really"),
    CreateApplicationCommand::new("awkward",  "Feeling awkward"),
    CreateApplicationCommand::new("oops",     "This is just oops emotion..."),
    CreateApplicationCommand::new("lazy",     "Feeling lazy"),
    CreateApplicationCommand::new("hungry",   "Feeling hungry"),
    CreateApplicationCommand::new("stressed", "Feeling stressed"),
    CreateApplicationCommand::new("scared",   "Really scared"),
    CreateApplicationCommand::new("bored",    "Feeling bored"),
    CreateApplicationCommand::new("yes",      "Yes Yes Yes"),
    CreateApplicationCommand::new("no",       "No No No"),
    CreateApplicationCommand::new("bye",      "Bye Bye"),
    CreateApplicationCommand::new("sorry",    "I am so sorry"),
    CreateApplicationCommand::new("sleepy",   "Feeling sleepy zzz"),
    CreateApplicationCommand::new("wink",     "Close and open one eye quickly"),
    CreateApplicationCommand::new("facepalm", "A palm of a hand is brought to a face as an expression of dismay"),
    CreateApplicationCommand::new("whatever", "you don't care"),
    CreateApplicationCommand::new("pout",     "do weird thing with lips"),
    CreateApplicationCommand::new("smug",     "showing an excessive pride in oneself"),
    CreateApplicationCommand::new("smirk",    "smile in an irritatingly smug, conceited, or silly way"),
    CreateApplicationCommand::new("hug",      "Literally hug someone")
      .add_option(CreateApplicationCommandOption::new(CommandOptionType::String, "person", "Person to hug")
          .required(true)
      ),
    CreateApplicationCommand::new("pat",    "Literally pat someone")
      .add_option(CreateApplicationCommandOption::new(CommandOptionType::String, "person", "Person to pat")
          .required(true)
      ),
    CreateApplicationCommand::new("slap",   "Literally slap someone")
      .add_option(CreateApplicationCommandOption::new(CommandOptionType::String, "person", "Person to slap")
          .required(true)
      ),
    CreateApplicationCommand::new("gif",    "Do some specific animation")
      .add_option(CreateApplicationCommandOption::new(CommandOptionType::String, "animation", "Search for specific animation")
          .required(true)
      ),
    CreateApplicationCommand::new("translate", "Translate Russian to English")
      .add_option(CreateApplicationCommandOption::new(CommandOptionType::String, "text", "What will be translated")
          .required(true)
      ),
    CreateApplicationCommand::new("перевод", "Перевод с английского на Русский")
      .add_option(CreateApplicationCommandOption::new(CommandOptionType::String, "текст", "Текст для перевода")
          .required(true)
      ),
    CreateApplicationCommand::new("stats",  "Display W3C player statistics")
      .add_option(CreateApplicationCommandOption::new(CommandOptionType::String, "battletag", "Target player")
          .required(true)
      ),
    CreateApplicationCommand::new("борис",  "...")
      .add_option(CreateApplicationCommandOption::new(CommandOptionType::String, "текст", "Текст для Бориса")
          .required(true)
      ),
    CreateApplicationCommand::new("uwu",    "Uwufy some text OwO")
      .add_option(CreateApplicationCommandOption::new(CommandOptionType::String, "text", "Some text...")
          .required(true)
      ),
    CreateApplicationCommand::new("феминизировать", "Феминизировать предложение")
      .add_option(CreateApplicationCommandOption::new(CommandOptionType::String, "текст", "Текст для феминизации")
          .required(true)
      ),
    CreateApplicationCommand::new("time",   "Display current time")
      .add_option(CreateApplicationCommandOption::new(CommandOptionType::String, "timezone", "Optional timezone")
          .required(false)
      ),
    CreateApplicationCommand::new("время",  "Показать текущее время")
      .add_option(CreateApplicationCommandOption::new(CommandOptionType::String, "город", "Дополнительный часовой пояс")
          .required(false)
      ),
    CreateApplicationCommand::new("join",   "Join voice channel with you (you should be in voice channel)"),
    CreateApplicationCommand::new("leave",  "Leave voice channel"),
    CreateApplicationCommand::new("repeat", "Play last song again"),
    CreateApplicationCommand::new("play",   "Play radio stream or youtube stuff")
      .add_option(CreateApplicationCommandOption::new(CommandOptionType::String, "url", "link for music to play")
          .required(true)
      )
    ]
  ).await {
    error!("Failed to register global application commands {why}");
  }
}
