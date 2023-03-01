use abby_utils::{
	Error,
	db_structs
};
use poise::serenity_prelude as serenity;
use regex::Regex;

// Public handler function for module.
pub async fn handler(ctx: &serenity::Context, msg: &serenity::Message, srv_features: db_structs::Server) -> Result<(), Error> {
	let content = msg.content.as_str();
	if !srv_features.serious {
		borger(ctx, msg, content).await?;
		v(ctx, msg, content).await?;
	}
	Ok(())
}

async fn borger(ctx: &serenity::Context, msg: &serenity::Message, content: &str) -> Result<(), Error> {
  if Regex::new(r"(?i)\S*b[ou]rger\b").unwrap().is_match(content) {
    let time = time::OffsetDateTime::now_utc()
      .to_offset(time::UtcOffset::from_hms(-5, 0, 0)?)
      .format(&time::format_description::parse("[hour repr:12 padding:none]:[minute] [period case:upper]")?)?;
    msg.reply(ctx, format!("It is now {time} in Borger, Texas.")).await?;
  }
  Ok(())
}

async fn v(ctx: &serenity::Context, msg: &serenity::Message, content: &str) -> Result<(), Error> {
  if Regex::new(r"(?i)\S*vore\b").unwrap().is_match(content) {
		msg.reply(ctx, "https://i.imgur.com/59urJXr.png").await?;
  }
  Ok(())
}
