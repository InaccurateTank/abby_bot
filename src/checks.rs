use color_eyre::Result;
use crate::{
	database, templates, Context
};

fn feature_not_enabled(
	feature: &str
) -> poise::CreateReply {
	poise::CreateReply::default()
		.embed(
			templates::status::warning(
				Some("Feature Not Enabled"),
				format!("The {feature:?} feature is not enabled in this server.")
			)
		).reply(true)
		.ephemeral(true)
}

pub async fn admin(ctx: Context<'_>) -> Result<bool> {
	// Admin commands are guild exclusive. They shouldn't even be *registered* outside of one.
	let Some(id) = ctx.guild_id() else {
		return Ok(false)
	};
	match database::CheckGuildSetting::Admin.query(&id, &ctx.data().db).await {
		Ok(v) => if !v {
			ctx.send(feature_not_enabled("roles")).await?;
			return Ok(false)
		},
		Err(e) => return Err(e)
	};
	Ok(true)
}

pub async fn roles(ctx: Context<'_>) -> Result<bool> {
	// Role management is a server exclusive feature. Somthing is deeply wrong if we're not in a guild.
	let Some(id) = ctx.guild_id() else {
		return Ok(false)
	};
	match database::CheckGuildSetting::Role.query(&id, &ctx.data().db).await {
		Ok(v) => if !v {
			ctx.send(feature_not_enabled("roles")).await?;
			return Ok(false)
		},
		Err(e) => return Err(e)
	};
	Ok(true)
}

pub async fn unserious(ctx: Context<'_>) -> Result<bool> {
	// Bypass check if it's in a DM. It shouldn't be there anyway, but who cares if it is.
	let Some(id) = ctx.guild_id() else {
		return Ok(true)
	};
	match database::CheckGuildSetting::Unserious.query(&id, &ctx.data().db).await {
		Ok(v) => 	if !v {
			ctx.send(feature_not_enabled("unserious")).await?;
			return Ok(false)
		},
		Err(e) => return Err(e)
	};
	Ok(true)
}
