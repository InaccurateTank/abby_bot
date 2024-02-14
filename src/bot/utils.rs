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
	ctx: impl serenity::CacheHttp + Copy,
	guild: serenity::Guild,
	botuser: serenity::UserId
) -> Option<serenity::ChannelId> {
	// Get the channel
	let channel = if let Some(system_channel) = guild.system_channel_id {
		Some(system_channel)
	} else {
		guild.default_channel(botuser)
			.map(|f| f.id)
	};
	// If a channel was returned at all, which there should be but you never really know?
	if let Some(c) = channel {
		// Can messages even be sent in the channel?
		if can_post(ctx, c, botuser).await || c.to_channel(ctx).await.unwrap().guild().unwrap().kind == serenity::ChannelType::Text {
			// If so return it
			return Some(c)
		}
	}
	// Default return None
	None
}

/// Detects if a user can post to the provided [`serenity::Channel`]
pub async fn can_post(
	ctx: impl serenity::CacheHttp + Copy,
	chid: serenity::ChannelId,
	user: serenity::UserId
) -> bool {
	chid.to_channel(ctx)
		.await
		.unwrap()
		.guild()
		.unwrap()
		.permissions_for_user(ctx.cache().unwrap(), user)
		.unwrap()
		.send_messages()
}
