use poise::serenity_prelude as serenity;
use crate::{Context, Error, templates};

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
	Voice
}

async fn autocomplete_roles<'a>(
	ctx: Context<'_>,
	partial: &'a str
) -> Vec<String> {
	ctx.guild()
		.unwrap()
		.roles
		.into_iter()
		.filter_map(|(_, r)| {
			if r.name.to_lowercase().starts_with(partial) {
				return Some(r.name)
			}
			None
		}).collect::<Vec<String>>()
}

async fn autocomplete_category<'a>(
	ctx: Context<'_>,
	partial: &'a str
) -> Vec<String> {
	ctx.guild()
		.unwrap()
		.channels
		.into_iter()
		.filter_map(|(_, c)| {
			if let Some(cat) = c.category() {
				if cat.name.starts_with(partial) {
					return Some(cat.name)
				}
			}
			None
		}).collect::<Vec<String>>()
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
	#[description = "Role to be used as a filter for private channel purposes."]
	#[autocomplete = "autocomplete_roles"]
	role_filter: Option<String>,
	#[description = "Category for the new channel to go under."]
	#[autocomplete = "autocomplete_category"]
	category: Option<String>
) -> Result<(), Error> {
	// Find role if provided
	let role = role_filter.clone().and_then(|f| {
		if f == "@everyone" {
			return None
		}
		ctx.guild().unwrap().role_by_name(&f).cloned()
	});
	if role_filter.is_some() && role.is_none() {
		// Break early if doesn't exist
		ctx.send(|m| {
			m.content("");
			m.ephemeral(true);
			m.embed(|e| {
				templates::builder_state_embed(e, false, &format!("Role {} either does not exist or can't be used.", role_filter.unwrap()));
				e
			})
		}).await?;
		return Ok(())
	}

	// Find category if provided
	let category_id = category.clone().and_then(|f| {
		ctx.guild()
			.unwrap()
			.channels
			.into_iter()
			.find_map(|(channel_id, channel)| {
				if let Some(cat) = channel.category() {
					if cat.name == f {
						return Some(channel_id)
					}
				}
				None
			})
	});
	if category.is_some() && category_id.is_none() {
		// Break early if doesn't exist
		ctx.send(|m| {
			m.content("");
			m.ephemeral(true);
			m.embed(|e| {
				templates::builder_state_embed(e, false, &format!("Category {} does not exist.", category.unwrap()));
				e
			})
		}).await?;
		return Ok(())
	}

	ctx.guild_id().unwrap().create_channel(ctx, |f| {
		f.name(&name);
		// Voice or Text?
		match kind {
			ChannelKind::Text => f.kind(serenity::ChannelType::Text),
			ChannelKind::Voice => f.kind(serenity::ChannelType::Voice)
		};

		// Add a role filter to the channel
		if let Some(r) = role {
			let everyone = ctx.guild()
				.unwrap()
				.roles
				.into_iter()
				.find_map(|(_, r)| {
					if r.name == "@everyone" {
						return Some(r)
					}
					None
				})
				.unwrap();
			let permissions = vec![
				serenity::PermissionOverwrite {
					allow: serenity::Permissions::default(),
					deny: serenity::Permissions::VIEW_CHANNEL,
					kind: serenity::PermissionOverwriteType::Role(serenity::RoleId(*everyone.id.as_u64()))
				},
				serenity::PermissionOverwrite {
					allow: serenity::Permissions::VIEW_CHANNEL,
					deny: serenity::Permissions::empty(),
					kind: serenity::PermissionOverwriteType::Role(serenity::RoleId(*r.id.as_u64()))
				}
			];
			f.permissions(permissions);
		}
		// Put the channel under a category
		if let Some(c) = category_id {
			f.category(c);
		}
		f
	}).await?;
	// Send interaction reply.
	ctx.send(|m| {
		m.content("");
		m.ephemeral(true);
		m.embed(|e| {
			templates::builder_state_embed(e, true, &format!("Channel {} has been successfully added to the server.", name));
			e
		})
	}).await?;
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
	if ctx.guild()
		.unwrap()
		.roles
		.values()
		.find(|f| {
			f.name.to_lowercase() == name
		}).is_some() {
		ctx.send(|m| {
			m.content("");
			m.ephemeral(true);
			m.embed(|e| {
				templates::builder_state_embed(e, false, "Role with identical name already exists.");
				e
			})
		}).await?;
		return Ok(())
	}
	ctx.guild_id()
		.unwrap()
		.create_role(ctx, |f| {
		f.name(&name)
	}).await?;
	ctx.send(|m| {
		m.content("");
		m.ephemeral(true);
		m.embed(|e| {
			templates::builder_state_embed(e, true, &format!("Role {name} has been successfully created."));
			e
		})
	}).await?;
	Ok(())
}
