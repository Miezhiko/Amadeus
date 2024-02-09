use serenity::{
  prelude::*,
  builder::*,
  gateway::ActivityData,
  model::id::ChannelId
};

use tokio::process::Command;

use regex::Regex;
use once_cell::sync::Lazy;

pub async fn upgrade_amadeus(ctx: &Context, channel_id: ChannelId) -> anyhow::Result<()> {
  let start_typing = ctx.http.start_typing(channel_id);
  ctx.set_activity(Some( ActivityData::listening("Fetching changes") ));
  ctx.idle();
  let git_fetch = Command::new("sh")
                  .arg("-c").arg("git fetch origin mawa")
                  .output()
                  .await
                  .expect("failed to execute git fetch");
  let _git_adak = Command::new("sh")
                  .arg("-c").arg("git add aka.yml")
                  .output()
                  .await
                  .expect("failed add aka.yml");
  let _git_cmak = Command::new("sh")
                  .arg("-c").arg("git commit -m \"aka database update\"")
                  .output()
                  .await.unwrap(); // ignore error (if aka db not changed)
  let _gprbwtfk = Command::new("sh")
                  .arg("-c").arg("git pull --rebase origin mawa")
                  .output()
                  .await.unwrap(); // ignore error (if aka db not changed)
  let _git_push = Command::new("sh")
                  .arg("-c").arg("git push origin mawa")
                  .output()
                  .await.unwrap(); // ignore error (if aka db not changed)
  let git_reset = Command::new("sh")
                  .arg("-c").arg("git reset --hard origin/mawa")
                  .output()
                  .await
                  .expect("failed to reset on remote branch");
  if let Ok(git_fetch_out) = &String::from_utf8(git_fetch.stdout) {
    if let Ok(git_reset_out) = &String::from_utf8(git_reset.stdout) {
      let mut description = format!("{git_fetch_out}\n{git_reset_out}");
      let mut mmm = channel_id.send_message(&ctx, CreateMessage::new()
        .embed(CreateEmbed::new()
                .title("Updating")
                .colour((220, 20, 100))
                .description(&description)
        )
      ).await?;
      ctx.set_activity(Some( ActivityData::playing("Compiling...") ));
      static LINKS_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"(.https.*)").unwrap());
      let cargo_update = Command::new("sh")
                .arg("-c").arg("cargo update")
                .output()
                .await
                .expect("failed to update crates");
      if let Ok(cargo_update_out) = &String::from_utf8(cargo_update.stderr) {
        static GIT_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"(.Updating git.*)").unwrap());
        let mut update_str = LINKS_RE.replace_all(cargo_update_out, "").to_string();
        update_str = GIT_RE.replace_all(&update_str, "").to_string();
        update_str = update_str.replace("/data2/contrib/rust/", "");
        update_str = update_str.lines()
                               .filter(|l| !l.trim().is_empty())
                               .collect::<Vec<&str>>()
                               .join("\n");
        if update_str.len() > 666 {
          if let Some((i, _)) = update_str.char_indices().rev().nth(666) {
            update_str = update_str[i..].to_string();
          }
        }
        description = format!("{}\n{update_str}", &description);
        mmm.edit(&ctx, EditMessage::default()
          .embed(CreateEmbed::new().title("Compiling")
                                       .colour((230, 10, 50))
                                       .description(&description)
          )
        ).await?;
      }

      let _git_adcl2 = Command::new("sh")
            .arg("-c").arg("git add Cargo.lock")
            .output()
            .await
            .expect("failed add Cargo.lock");
      let _git_cmak2 = Command::new("sh")
            .arg("-c").arg("git commit -m \"some updates\"")
            .output()
            .await.unwrap(); // ignore error (if aka db not changed)
      let _git_push2 = Command::new("sh")
            .arg("-c").arg("git push origin mawa")
            .output()
            .await.unwrap(); // ignore error (if aka db not changed)

      let cargo_build = Command::new("sh")
                .arg("-c").arg("hake")
                .output()
                .await
                .expect("failed to compile new version");
      if let Ok(cargo_build_out) = &String::from_utf8(cargo_build.stderr) {
        let mut cut_paths = cargo_build_out.replace("/data2/contrib/rust/", "");
        cut_paths = LINKS_RE.replace_all(&cut_paths, "").to_string();
        // if message is too big, take only last things
        if cut_paths.len() > 666 {
          if let Some((i, _)) = cut_paths.char_indices().rev().nth(666) {
            cut_paths = cut_paths[i..].to_string();
          }
        }
        description = format!("{}\n{cut_paths}", &description);
        mmm.edit(&ctx, EditMessage::default()
          .embed(CreateEmbed::new().title("Upgrading")
                                       .colour((250, 0, 0))
                                       .description(&description)
          )
        ).await?;
        ctx.set_activity(Some( ActivityData::listening("Restarting") ));
        let _systemctl1 = Command::new("sh")
                .arg("-c").arg("sudo systemctl restart Vivaldi")
                .output()
                .await
                .expect("failed to restart Vivaldi service");
        let _systemctl2 = Command::new("sh")
                .arg("-c").arg("sudo systemctl restart Salieri")
                .output()
                .await
                .expect("failed to restart Salieri service");
        let _systemctl3 = Command::new("sh")
                .arg("-c").arg("sudo systemctl restart Amadeus")
                .output()
                .await
                .expect("failed to restart Amadeus service");
        // I expect that we die right here
      }
    }
  }
  start_typing.stop();
  Ok(())
}
