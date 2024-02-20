use poise::serenity_prelude::{self as serenity, Mentionable};
use crate::{
	templates,
	Context, Error
};

/// A set of commands to quickly add things to the server. Less powerful than discords standard options.
///
/// ```Subcommands:
///  channel  Adds a channel to the server, with optional role lock.
///  role     Adds a role to the server that is guaranteed to be below the bot in permissions.
/// ```
/// All subcommands are ephemeral and require the `Manage Server` permission.
#[poise::command(
	guild_only,
	slash_command,
	required_permissions="MANAGE_GUILD",
	category="Administration",
	ephemeral,
	subcommands("channel", "role")
)]
pub async fn snap(ctx: Context<'_>) -> Result<(), Error> {
	ctx.say("You shouldn't be here?").await?;
	Ok(())
}

#[derive(Debug, poise::ChoiceParameter)]
enum ChannelKind {
	Text,
	Voice,
	Forum
}

/// Adds a new channel to the server.
#[poise::command(
	guild_only,
	slash_command,
	required_permissions="MANAGE_GUILD",
	ephemeral
)]
async fn channel(
	ctx: Context<'_>,
	#[description = "Name of the new channel."]
	name: String,
	#[description = "Kind of channel."]
	kind: ChannelKind,
	#[description = "Initial role to be used as a filter for visibility purposes."]
	role_filter: Option<serenity::RoleId>,
	#[description = "Category for the new channel to go under."]
	#[channel_types("Category")]
	category: Option<serenity::ChannelId>
) -> Result<(), Error> {
	let guild_id = ctx.guild_id().unwrap();

	let mut builder = serenity::CreateChannel::new(&name)
	// Voice or Text?
	.kind(
		match kind {
			ChannelKind::Text => serenity::ChannelType::Text,
			ChannelKind::Voice => serenity::ChannelType::Voice,
			ChannelKind::Forum => serenity::ChannelType::Forum
		}
	);

	// Add a role filter to the channel
	if let Some(r) = role_filter {
		let everyone = guild_id.everyone_role();
		builder = builder.permissions(vec![
			serenity::PermissionOverwrite {
				allow: serenity::Permissions::default(),
				deny: serenity::Permissions::VIEW_CHANNEL,
				kind: serenity::PermissionOverwriteType::Role(everyone)
			},
			serenity::PermissionOverwrite {
				allow: serenity::Permissions::VIEW_CHANNEL,
				deny: serenity::Permissions::empty(),
				kind: serenity::PermissionOverwriteType::Role(r)
			}
		]);
	}

	// Put the channel under a category
	if let Some(c) = category {
		builder = builder.category(c);
	}

	// Send interaction reply.
	match guild_id.create_channel(ctx, builder).await {
		Ok(c) => {
			ctx.send(poise::CreateReply::default()
				.ephemeral(true)
				.embed(templates::state_embed(true, &format!("Channel {} has been successfully added to the server.", c.mention())))
			).await?;
		}
		Err(e) => {
			ctx.send(poise::CreateReply::default()
				.ephemeral(true)
				.embed(templates::state_embed(false, &format!("Error adding channel {name}: {e:?}")))
			).await?;
		}
	};
	Ok(())
}

/// Adds a new role to the server.
#[poise::command(
	guild_only,
	slash_command,
	required_permissions="MANAGE_GUILD",
	ephemeral
)]
async fn role(
	ctx: Context<'_>,
	#[description = "Name of the new role."]
	name: String
) -> Result<(), Error> {
	if ctx.guild().unwrap().role_by_name(&name).is_some() {
		ctx.send(poise::CreateReply::default()
			.ephemeral(true)
			.embed(templates::state_embed(false, "Role with an identical name already exists."))
		).await?;
		return Ok(())
	}

	match ctx.guild_id()
		.unwrap()
		.create_role(ctx, serenity::EditRole::new()
			.name(&name)
		).await {
		Ok(r) => {
			ctx.send(poise::CreateReply::default()
				.ephemeral(true)
				.embed(templates::state_embed(true, &format!("Role {} has been successfully created.", r.mention())))
			).await?;
		},
		Err(e) => {
			ctx.send(poise::CreateReply::default()
				.ephemeral(true)
				.embed(templates::state_embed(false, &format!("Error creating role {name}: {e:?}")))
			).await?;
		}
	}
	Ok(())
}
