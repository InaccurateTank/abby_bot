use color_eyre::Result;
use crate::{
	Context,
	templates
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

pub async fn unserious(ctx: Context<'_>) -> Result<bool> {
	// Bypass check if it's in a DM. It shouldn't be there anyway, but who cares if it is.
	let Some(id) = ctx.guild_id() else {
		return Ok(true)
	};
	let unserious = sqlx::query_scalar::<_, bool>("SELECT unserious FROM guild_settings WHERE guild_id = ?;")
		.bind(id.get() as i64)
		.fetch_one(&ctx.data().db)
		.await;
	match unserious {
		Ok(v) => 	if !v {
			ctx.send(feature_not_enabled("unserious")).await?;
			return Ok(false)
		},
		Err(e) => return Err(e.into())
	};
	// if !unserious {
	// 	ctx.send(feature_not_enabled("unserious")).await?;
	// 	return Ok(false)
	// }
	Ok(true)
}

pub async fn roles(ctx: Context<'_>) -> Result<bool> {
	// Role management is a server exclusive feature. Somthing is deeply wrong if we're not in a guild.
	let Some(id) = ctx.guild_id() else {
		return Ok(false)
	};
	let roles = sqlx::query_scalar::<_, bool>("SELECT roles FROM guild_settings WHERE guild_id = ?;")
		.bind(id.get() as i64)
		.fetch_one(&ctx.data().db)
		.await;
	match roles {
		Ok(v) => if !v {
			ctx.send(feature_not_enabled("roles")).await?;
			return Ok(false)
		},
		Err(e) => return Err(e.into())
	};
	// println!("Role Check: {roles}");
	// if !roles {
	// 	ctx.send(feature_not_enabled("roles")).await?;
	// 	return Ok(false)
	// }
	Ok(true)
}
