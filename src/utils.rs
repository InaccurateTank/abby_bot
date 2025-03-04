//! Various utility functions that don't fit into any other module.

use color_eyre::Result;
use poise::serenity_prelude as serenity;
use crate::Context;

/// Combines two [str] into a [String].
pub fn concat(a: &str, b: &str) -> String {
  let mut result: String = String::with_capacity(a.len() + b.len());
  result += a;
  result += b;
  result
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
	guild: &serenity::Guild,
	bot_id: serenity::UserId
) -> Result<Option<serenity::ChannelId>> {
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
) -> Result<bool> {
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

/// Converts the [Option] containing a guild [CacheRef][serenity::CacheRef] into either an owned [Guild][serenity::Guild] or [Error][serenity::Error].
pub fn guild_or_error(ctx: Context<'_>) -> Result<serenity::Guild> {
	let Some(guild) = ctx.guild() else {
		return Err(serenity::Error::Model(serenity::ModelError::GuildNotFound).into())
	};
	Ok(guild.to_owned())
}
