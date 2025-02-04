use poise::serenity_prelude as serenity;
use crate::{Context, Error};

pub fn concat(a: &str, b: &str) -> String {
  let mut result: String = String::with_capacity(a.len() + b.len());
  result += a;
  result += b;
  result
}

pub async fn feature_not_enabled(ctx: Context<'_>) -> Result<(), Error> {
	ctx.send(poise::CreateReply::default()
		.content("")
		.embed(serenity::CreateEmbed::default()
			.title(":warning: Failure :warning:")
			.color(serenity::Color::from_rgb(178, 34, 34))
			.description("This command requires a feature that isn't enabled on the server. If this is a mistake, have an admin run `/setup bot` to change it.")
		)
	).await?;
	Ok(())
}

/// Abby should never assign moderation roles and should never show @everyone.
///
/// Returns [true] when the role is safe to assign.
pub fn role_filter(role: &serenity::Role) -> bool {
	if role.has_permission(serenity::Permissions::ADMINISTRATOR)
	|| role.has_permission(serenity::Permissions::MANAGE_CHANNELS)
	|| role.has_permission(serenity::Permissions::MANAGE_GUILD_EXPRESSIONS)
	|| role.has_permission(serenity::Permissions::MANAGE_EVENTS)
	|| role.has_permission(serenity::Permissions::MANAGE_GUILD)
	|| role.has_permission(serenity::Permissions::MANAGE_MESSAGES)
	|| role.has_permission(serenity::Permissions::MANAGE_NICKNAMES)
	|| role.has_permission(serenity::Permissions::MANAGE_ROLES)
	|| role.has_permission(serenity::Permissions::MANAGE_THREADS)
	|| role.has_permission(serenity::Permissions::MANAGE_WEBHOOKS) {
		return false
	}
	if role.name == "@everyone" {
		return false
	}
	true
}

/// Selects either the system messages channel or, if that isn't an option, the default channel. Returns [`None`] if the channel isn't writeable.
pub async fn default_bot_channel(
	ctx: impl serenity::CacheHttp + Copy + AsRef<serenity::Cache>,
	guild: serenity::Guild,
	bot_id: serenity::UserId
) -> Result<Option<serenity::ChannelId>, Error> {
	let Some(channel) = ({
		match guild.system_channel_id {
			// Use system channel if available
			Some(system_channel) => Some(system_channel),
			// Check for a default channel otherwise
			None => guild.default_channel(bot_id).map(|f| f.id)
		}
	}) else {
		// None means none
		return Ok(None)
	};
	// Check if the bot can post to the channel
	match can_post(ctx, &channel, bot_id).await? {
		true => Ok(Some(channel)),
		false => Ok(None)
	}
}

/// Checks if a user can post to the provided [`serenity::Channel`]
pub async fn can_post(
	ctx: impl serenity::CacheHttp + Copy + AsRef<serenity::Cache>,
	chid: &serenity::ChannelId,
	user: serenity::UserId
) -> Result<bool, Error> {
	let Some(channel) = chid.to_channel(ctx).await?.guild() else {
		// If the channel has no guild it's a private channel.
		return Ok(true)
	};
	let guild = channel.guild(&ctx)
		// Frankly if the explicitly filtered guild channel has no guild, we have bigger issues.
		.unwrap()
		.to_owned();
	let member = guild.member(ctx, user).await?;
	if !guild.user_permissions_in(&channel, &member).send_messages() {
		return Ok(false)
	}
	Ok(true)
}

/// Poise command checker for server features. Checks if the server should have meme features.
pub async fn check_serious(ctx: Context<'_>) -> Result<bool, Error> {
	// Bypass check if it's in a DM. It shouldn't be there anyway, but who cares if it is.
	let Some(id) = ctx.guild_id() else {
		return Ok(true)
	};
	let serious = sqlx::query_scalar::<_, bool>("SELECT serious FROM server_settings WHERE id = ?);")
		.bind(id.get() as i64)
		.fetch_one(&ctx.data().db)
		.await?;
	if serious {
		feature_not_enabled(ctx).await?;
		return Ok(false)
	}
	Ok(true)
}

/// Poise command checker for server features. Checks if the server allows the bot to manage roles.
pub async fn check_roles(ctx: Context<'_>) -> Result<bool, Error> {
	// Role management is a server exclusive feature. Somthing is deeply wrong if we're not in a guild.
	let Some(id) = ctx.guild_id() else {
		return Ok(false)
	};
	let roles = sqlx::query_scalar::<_, bool>("SELECT roles FROM server_settings WHERE id = ?);")
		.bind(id.get() as i64)
		.fetch_one(&ctx.data().db)
		.await?;
	if roles {
		feature_not_enabled(ctx).await?;
		return Ok(false)
	}
	Ok(true)
}
