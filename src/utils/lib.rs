use poise::serenity_prelude as serenity;

// Setup
pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;

// Data available in all command invocations.
pub struct Data {
	pub db: sqlx::Pool<sqlx::Sqlite>
}

pub mod config;
pub use config::Config;
pub mod db_structs;

fn stamp() -> Result<String, Error>{
	if let Ok(r) = time::OffsetDateTime::now_local() {
		if let Ok(f) = time::format_description::parse("[year]-[month]-[day]T[hour]:[minute]:[second][offset_hour sign:mandatory]:[offset_minute]") {
			if let Ok(ts) = r.format(&f) {
				return Ok(ts)
			}
		}
	}
	Err("Could not get time".into())
}

pub async fn cmd_err(reply: &str, term: &str, ctx: Context<'_>, msg: poise::ReplyHandle<'_>) -> Result<(), Error> {
	msg.edit(ctx, |b| {
			b.components(|b| b).content(reply)
		})
		.await?;
	if !term.is_empty() {
		eprintln!("{} - {} in \"{}\"",
			stamp()?,
			term,
			ctx.guild().unwrap().name);
	}
	Ok(())
}

pub async fn inter_err(reply: &str, term: &str, ctx: &serenity::Context, int: &serenity::MessageComponentInteraction) -> Result<(), Error> {
	int.create_interaction_response(ctx, |f| {
		f.kind(serenity::InteractionResponseType::ChannelMessageWithSource);
		f.interaction_response_data(|m| {
			m.content(reply);
			m.ephemeral(true)
		})
	}).await?;
	if !term.is_empty() {
		eprintln!("{} - {} in \"{}\"",
			stamp()?,
			term,
			int.guild_id.unwrap().name(&ctx.cache).unwrap());
	}
	Ok(())
}

pub fn concat(a: &str, b: &str) -> String {
  let mut result: String = String::with_capacity(a.len() + b.len());
  result += a;
  result += b;
  result
}

pub async fn feature_not_enabled(ctx: Context<'_>) -> Result<(), Error> {
	ctx.send(|r| {
		r.content("");
		r.embed(|e|{
			e.title(":warning: Failure :warning:");
			e.color(serenity::utils::Color::from_rgb(178, 34, 34));
			e.description("This command requires a feature that isn't enabled on the server. If this is a mistake, have an admin run `/setup bot` to change it.")
		})
	}).await?;
	Ok(())
}
