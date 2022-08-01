use serenity::{
  prelude::*,
  builder::{ CreateApplicationCommand, CreateApplicationCommandOption },
  model::{ guild::PartialGuild
         , application::command::CommandOptionType
         }
};

pub async fn create_app_commands(ctx: &Context, guild: &PartialGuild) {
  if let Err(why) = guild.set_application_commands(ctx, vec![
    CreateApplicationCommand::default().name("help")
      .description("Display Amadeus Help"),
    CreateApplicationCommand::default().name("wave")
      .description("Wave a hand you know..."),
    CreateApplicationCommand::default().name("cry")
      .description("Start to cry!"),
    CreateApplicationCommand::default().name("cringe")
      .description("just cringe"),
    CreateApplicationCommand::default().name("ahegao")
      .description("Make an ahegao face"),
    CreateApplicationCommand::default().name("clap")
      .description("Start clapping"),
    CreateApplicationCommand::default().name("shrug")
      .description("Shrug shoulders"),
    CreateApplicationCommand::default().name("lol")
      .description("laugh out loud"),
    CreateApplicationCommand::default().name("angry")
      .description("Angry feels"),
    CreateApplicationCommand::default().name("dance")
      .description("Dance Dance Dance"),
    CreateApplicationCommand::default().name("confused")
      .description("Shows your confusion"),
    CreateApplicationCommand::default().name("shock")
      .description("If you are shocked"),
    CreateApplicationCommand::default().name("nervous")
      .description("Feeling nervous"),
    CreateApplicationCommand::default().name("sad")
      .description("Feeling sad"),
    CreateApplicationCommand::default().name("happy")
      .description("Feeling happy"),
    CreateApplicationCommand::default().name("annoyed")
      .description("Really annoyed"),
    CreateApplicationCommand::default().name("omg")
      .description("Oh my gawd"),
    CreateApplicationCommand::default().name("smile")
      .description("Do a smile"),
    CreateApplicationCommand::default().name("ew")
      .description("When you don't like something really"),
    CreateApplicationCommand::default().name("awkward")
      .description("Feeling awkward"),
    CreateApplicationCommand::default().name("oops")
      .description("This is just oops emotion..."),
    CreateApplicationCommand::default().name("lazy")
      .description("Feeling lazy"),
    CreateApplicationCommand::default().name("hungry")
      .description("Feeling hungry"),
    CreateApplicationCommand::default().name("stressed")
      .description("Feeling stressed"),
    CreateApplicationCommand::default().name("scared")
      .description("Really scared"),
    CreateApplicationCommand::default().name("bored")
      .description("Feeling bored"),
    CreateApplicationCommand::default().name("yes")
      .description("Yes Yes Yes"),
    CreateApplicationCommand::default().name("no")
      .description("No No No"),
    CreateApplicationCommand::default().name("bye")
      .description("Bye Bye"),
    CreateApplicationCommand::default().name("sorry")
      .description("I am so sorry"),
    CreateApplicationCommand::default().name("sleepy")
      .description("Feeling sleepy zzz"),
    CreateApplicationCommand::default().name("wink")
      .description("Close and open one eye quickly"),
    CreateApplicationCommand::default().name("facepalm")
      .description("A palm of a hand is brought to a face as an expression of dismay"),
    CreateApplicationCommand::default().name("whatever")
      .description("you don't care"),
    CreateApplicationCommand::default().name("pout")
      .description("do weird thing with lips"),
    CreateApplicationCommand::default().name("smug")
      .description("showing an excessive pride in oneself"),
    CreateApplicationCommand::default().name("smirk")
      .description("smile in an irritatingly smug, conceited, or silly way"),
    CreateApplicationCommand::default().name("hug")
      .description("Literally hug someone")
      .add_option(CreateApplicationCommandOption::default()
          .name("person")
          .description("Person to hug")
          .kind(CommandOptionType::String)
          .required(true)
      ),
    CreateApplicationCommand::default().name("pat")
      .description("Literally pat someone")
      .add_option(CreateApplicationCommandOption::default()
          .name("person")
          .description("Person to pat")
          .kind(CommandOptionType::String)
          .required(true)
      ),
    CreateApplicationCommand::default().name("slap")
      .description("Literally slap someone")
      .add_option(CreateApplicationCommandOption::default()
          .name("person")
          .description("Person to slap")
          .kind(CommandOptionType::String)
          .required(true)
      ),
    CreateApplicationCommand::default().name("gif")
      .description("Do some specific animation")
      .add_option(CreateApplicationCommandOption::default()
          .name("animation")
          .description("Search for specific animation")
          .kind(CommandOptionType::String)
          .required(true)
      ),
    CreateApplicationCommand::default().name("translate")
      .description("Translate Russian to English")
      .add_option(CreateApplicationCommandOption::default()
          .name("text")
          .description("What will be translated")
          .kind(CommandOptionType::String)
          .required(true)
      ),
    CreateApplicationCommand::default().name("перевод")
      .description("Перевод с английского на Русский")
      .add_option(CreateApplicationCommandOption::default()
          .name("текст")
          .description("Текст для перевода")
          .kind(CommandOptionType::String)
          .required(true)
      ),
    CreateApplicationCommand::default().name("stats")
      .description("Display W3C player statistics")
      .add_option(CreateApplicationCommandOption::default()
          .name("battletag")
          .description("Target player")
          .kind(CommandOptionType::String)
          .required(true)
      ),
    CreateApplicationCommand::default().name("борис")
      .description("Команда, которую любит Лилуал")
      .add_option(CreateApplicationCommandOption::default()
          .name("текст")
          .description("Текст для Бориса")
          .kind(CommandOptionType::String)
          .required(true)
      ),
    CreateApplicationCommand::default().name("uwu")
      .description("Uwufy some text OwO")
      .add_option(CreateApplicationCommandOption::default()
          .name("text")
          .description("Some text...")
          .kind(CommandOptionType::String)
          .required(true)
      ),
    CreateApplicationCommand::default().name("феминизировать")
      .description("Феминизировать предложение")
      .add_option(CreateApplicationCommandOption::default()
          .name("текст")
          .description("Текст для феминизации")
          .kind(CommandOptionType::String)
          .required(true)
      ),
    CreateApplicationCommand::default().name("time")
      .description("Display current time")
      .add_option(CreateApplicationCommandOption::default()
          .name("timezone")
          .description("Optional timezone")
          .kind(CommandOptionType::String)
          .required(false)
      ),
    CreateApplicationCommand::default().name("время")
      .description("Показать текущее время")
      .add_option(CreateApplicationCommandOption::default()
          .name("город")
          .description("Дополнительный часовой пояс")
          .kind(CommandOptionType::String)
          .required(false)
      ),
    CreateApplicationCommand::default().name("join")
      .description("Join voice channel with you (you should be in voice channel)"),
    CreateApplicationCommand::default().name("leave")
      .description("Leave voice channel"),
    CreateApplicationCommand::default().name("repeat")
      .description("Play last song again"),
    CreateApplicationCommand::default().name("play")
      .description("Play radio stream or youtube stuff")
      .add_option(CreateApplicationCommandOption::default()
          .name("url")
          .description("link for music to play")
          .kind(CommandOptionType::String)
          .required(true)
      )
    ]
  ).await {
    error!("Failed to register global application commands {why}");
  }
}
