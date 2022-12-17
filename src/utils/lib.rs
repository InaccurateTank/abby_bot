use poise::serenity_prelude as serenity;
use time;

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;
pub struct Data {}

pub mod role_utils;
pub use role_utils::*;

fn stamp() -> Result<String, Error>{
	if let Ok(r) = time::OffsetDateTime::now_local() {
		if let Ok(f) = time::format_description::parse("[year]-[month]-[day]T[hour]:[minute]:[second][offset_hour sign:mandatory]:[offset_minute]") {
			if let Ok(ts) = r.format(&f) {
				return Ok(ts)
			}
		}
	}
	return Err("Could not get time".into())
}

pub async fn cmd_err(reply: &str, term: &str, ctx: Context<'_>, msg: poise::ReplyHandle<'_>) -> Result<(), Error> {
	msg.edit(ctx, |b| {
			b.components(|b| b).content(reply)
		})
		.await?;
	if term.is_empty() == false {
		eprintln!("{} - {} in \"{}\"",
			stamp()?,
			term,
			ctx.guild().unwrap().name);
	}
	Ok(())
}

pub async fn inter_err(reply: &str, term: &str, ctx: &serenity::Context, int: &serenity::MessageComponentInteraction) -> Result<(), Error> {
	int.create_interaction_response(ctx, |f| {
		f.kind(serenity::InteractionResponseType::UpdateMessage).interaction_response_data(|d| {
				d.components(|c| c).content(reply)
			})
	}).await?;
	if term.is_empty() == false {
		eprintln!("{} - {} in \"{}\"",
			stamp()?,
			term,
			int.guild_id.unwrap().name(ctx.cache.to_owned()).unwrap());
	}
	Ok(())
}
