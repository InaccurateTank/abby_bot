use color_eyre::Result;
use poise::serenity_prelude::{self as serenity, Mentionable};
use tracing::instrument;
use crate::{
	checks,
	colors,
	commands,
	database,
	error::{BotError, UserError},
	templates,
	utils,
	Context
};

/// Administrates the bot on a per-server basis.
///
/// ```Subcommands:
///  bot      Manages serverwide bot features.
///  roles    Manages the settings of the roles feature.
/// ```
/// All subcommands are ephemeral and require the `Manage Server` permission.
#[poise::command(
	guild_only,
	slash_command,
	required_permissions="MANAGE_GUILD",
	category="Administration",
	ephemeral,
	subcommands("roles", "bot")
)]
pub async fn setup(ctx: Context<'_>) -> Result<()> {
	ctx.say("You shouldn't be here?").await?;
	Ok(())
}

#[instrument(skip_all)]
/// Sets up various settings for the bot on the server.
#[poise::command(
	guild_only,
	slash_command,
	ephemeral
)]
async fn bot(ctx: Context<'_>) -> Result<()> {
	let guild = utils::guild_or_error(ctx)?;

	let guild_settings = database::GuildSettings::from_query(guild.id, &ctx.data().db).await?;

	let reply = ctx.send(poise::CreateReply::default()
		.embed(serenity::CreateEmbed::new()
			.title("Bot Configuration")
			.color(colors::INFO)
			.description("Below is a select menu to choose my features on a per-server basis. These can be changed at any time by simply running the command again.")
			.field("Admin", "Toggles the usability of all server administration commands. Off by default for security (through obscurity) reasons.", false)
			.field("Unserious", "Toggles the appearence of memes, injokes or other related content. This setting encompasses all invocations of this across all other features.", false)
			.field("Messages", "Whether or not messages should be listened to outside of the standard text command context. This is mostly to reply to them based on regex.", false)
			.field("Roles", "Allows the creation and management of role lists to allow members to self-assign roles.", false)
		).components(vec![
			serenity::CreateActionRow::SelectMenu(serenity::CreateSelectMenu::new("setup.bot", serenity::CreateSelectMenuKind::String { options: guild_settings.as_selectmenuoptions() })
				.placeholder("Please select features...")
				.min_values(0)
				.max_values(guild_settings.as_array().len() as u8))
		])
	).await?;

	let Some(interaction) = reply.message().await?
		.await_component_interaction(ctx)
		.author_id(ctx.author().id)
		.timeout(std::time::Duration::from_secs(300))
		.await else {
		// reply.edit(ctx, poise::CreateReply::default()
		// 	.embed(templates::state_embed(false, "Interaction timed out, please try again."))
		// 	.components(Vec::new())
		// ).await?;
		// return Ok(())
		reply.delete(ctx).await?;
		return Err(UserError(BotError::InteractionTimedOut.into()).into())
	};

	reply.edit(ctx, poise::CreateReply::default()
		.embed(templates::status::processing())
		// .embed(templates::processing_embed())
		.components(Vec::new())
	).await?;

	let mut updated = match &interaction.data.kind {
		serenity::ComponentInteractionDataKind::StringSelect { values } => {
			database::GuildSettings {
				admin: values.contains(&"enable_admin".to_string()),
				unserious: values.contains(&"enable_unserious".to_string()),
				messages: values.contains(&"enable_messages".to_string()),
				roles: values.contains(&"enable_roles".to_string()),
				roles_channel: guild_settings.roles_channel
			}
		},
		_ => {
			// interaction.create_response(ctx, serenity::CreateInteractionResponse::UpdateMessage(serenity::CreateInteractionResponseMessage::new()
			// 	.embed(templates::state_embed(false, "Somehow recieved wrong interaction, please report this."))
			// 	.components(Vec::new())
			// )).await?;
			// return Ok(());
			interaction.create_response(ctx, serenity::CreateInteractionResponse::Acknowledge).await?;
			reply.delete(ctx).await?;
			return Err(BotError::WrongInteraction.into())
		}
	};

	let mut updated_commands: Vec<poise::Command<crate::Data, color_eyre::Report>> = Vec::new();

	if updated.admin {
		updated_commands.extend(commands::admin_commands());
	}

	// Changed Serious
	if updated.unserious {
		updated_commands.extend(commands::unserious_commands());
	}

	// Changed Roles
	match updated.roles {
		// Enabled
		true => {
			// Changed from Disabled
			if !guild_settings.roles {
				// Find default channel
				let default = utils::default_bot_channel(ctx, &guild, ctx.framework().bot_id).await?;
				// Set roles channel in structure
				updated.roles_channel = default;
				// Send message
				ctx.send(poise::CreateReply::default()
					.embed(
						if let Some(channel) = default {
							// Default channel exists
							templates::status::info(
								None,
								format!("The current default channel for role lists is {}. If this is not desired, please run `/setup roles` now.", channel.mention())
							)
						} else {
							// Default channel does not exist
							templates::status::warning(
								None,
								"Could not setup a default channel for roles. Before creating any lists, please run `/setup roles`."
							)
						}
					)
				).await?;
			}
			updated_commands.extend(commands::role_commands());
		},
		// Changed from Enabled
		false if guild_settings.roles => {
			// Unset roles channel in structure
			updated.roles_channel = None;
			// For groups
			for grp in database::groups_from_query(guild.id, &ctx.data().db).await? {
				// Deleting messages
				if ctx.http()
					.delete_message(updated.roles_channel.unwrap(), grp.message_id, Some("Role management disabled, deleting groups."))
					.await
					.is_err() {
					ctx.send(poise::CreateReply::default()
						.embed(
							templates::status::warning(
								None,
								format!("Either can't find or can't delete the message for the role group {:?}. Entries will be removed from the database, but the message will need to be deleted manually.", grp.group_name)
							)
						)
					).await?;
				}
				// Purging managed roles
				sqlx::query("DELETE FROM roles WHERE guild_id = ? AND group_name = ?")
					.bind(guild.id.get() as i64)
					.bind(grp.group_name)
					.execute(&ctx.data().db)
					.await?;
			}
		},
		_ => {}
	}

	// Changed Unserious
	if updated.unserious {
		updated_commands.extend(commands::unserious_commands());
	}

	// Register Commands
	poise::builtins::register_in_guild(ctx, &updated_commands, guild.id).await?;

	// Update Database
	updated.update_entry(guild.id, &ctx.data().db).await?;

	// Confirm message
	interaction.create_response(ctx, serenity::CreateInteractionResponse::UpdateMessage(serenity::CreateInteractionResponseMessage::new()
		.embed(
			templates::status::success(
					Some("Server Settings Confirmed"),
					"Your new settings are:"
				).fields(updated.as_array()
				.map(|(name, value)|  (name[0..1].to_uppercase() + &name[1..], if value {"Enabled"} else {"Disabled"}, false))
			)
		)
		// .embed(serenity::CreateEmbed::new()
		// 	.title("Feature Changes Confirmed!")
		// 	.color(EMBED_STD)
		// 	.description("Your new settings are:")
		// 	.fields(updated.as_array()
		// 		.map(|(name, value)|  (name[0..1].to_uppercase() + &name[1..], if value {"Enabled"} else {"Disabled"}, false))
		// 	)
		// )
	)).await?;
	Ok(())
}

#[instrument(skip_all)]
/// Edits settings for the role management feature.
#[poise::command(
	guild_only,
	slash_command,
	check = "checks::roles",
	ephemeral
)]
async fn roles(ctx: Context<'_>) -> Result<()> {
	let guild = utils::guild_or_error(ctx)?;
	let roles_channel = database::roles_channel_query(guild.id, &ctx.data().db)
		.await?;

	// Channel select
	let reply = ctx.send(poise::CreateReply::default()
		.embed(serenity::CreateEmbed::new()
			.title("Role Setup")
			.color(colors::INFO)
			.description("Please select a channel for role management to take place in. This channel should be completely empty save for the role lists. All interactions done with me via this channel will be ephemeral, so there should end up being no clutter.")
		).components(vec![
			serenity::CreateActionRow::SelectMenu(
				serenity::CreateSelectMenu::new(
					"setup.roles",
					serenity::CreateSelectMenuKind::Channel {
						channel_types: Some(vec![serenity::ChannelType::Text]),
						default_channels: roles_channel.map(|c| vec![c])
					}
				)
					.placeholder("Select a Channel.")
					.min_values(0)
					.max_values(1)
			)
		])
	).await?;

	// Await interaction
	let Some(interaction) = reply.message().await?
		.await_component_interaction(ctx)
		.author_id(ctx.author().id)
		.timeout(std::time::Duration::from_secs(300))
		.await else {
			// reply.edit(ctx, poise::CreateReply::default()
			// 	.embed(templates::state_embed(false, "Interaction timed out, please try again."))
			// 	.components(Vec::new())
			// ).await?;
			// return Ok(())
			reply.delete(ctx).await?;
			return Err(UserError(BotError::InteractionTimedOut.into()).into())
		};

	// Processing message
	reply.edit(ctx, poise::CreateReply::default()
		.embed(templates::status::processing())
		// .embed(templates::processing_embed())
		.components(Vec::new())
	).await?;

	// Return either selected or default channel
	let selected = match &interaction.data.kind {
		serenity::ComponentInteractionDataKind::ChannelSelect { values } => {
			if let Some(ch) = values.first() {
				if utils::can_post(ctx, ch, ctx.framework().bot_id).await? {
					Some(*ch)
				} else {
					return Err(UserError(BotError::ChannelInaccessable.into()).into())
				}
			} else {
				utils::default_bot_channel(ctx, &guild, ctx.framework().bot_id).await?
			}
		}
		_ => {
			// interaction.create_response(ctx, serenity::CreateInteractionResponse::UpdateMessage(serenity::CreateInteractionResponseMessage::new()
			// 	.embed(templates::state_embed(false, "Somehow recieved wrong interaction, please report this."))
			// 	.components(Vec::new())
			// )).await?;
			// return Ok(());
			interaction.create_response(ctx, serenity::CreateInteractionResponse::Acknowledge).await?;
			reply.delete(ctx).await?;
			return Err(BotError::WrongInteraction.into())
		}
	};

	if let Some(chid) = selected {
		// Fetch old group messages
		let old_lists = database::groups_from_query(guild.id, &ctx.data().db).await?;

		// Migrates if there are role messages in the old channel
		if !old_lists.is_empty() {
			if let Some(c) = roles_channel {
				for entry in old_lists {
					// If the old message can't even be reached then no point trying anyway.
					if let Ok(old_message) = c.message(ctx, entry.message_id).await {
						let new_message = chid.send_message(ctx, serenity::CreateMessage::new()
							// Copy embed
							.embed(serenity::CreateEmbed::from(old_message.embeds.first().unwrap().to_owned()))
							// Add components
							.components(vec![
								templates::rolelist::components(&entry.group_name)
							])
						).await?;
						// Update database with new message
						sqlx::query("UPDATE role_groups SET group_message = ? WHERE guild_id = ? AND group_name = ?;")
							.bind(guild.id.get() as i64)
							.bind(new_message.id.get() as i64)
							.bind(entry.group_name)
							.execute(&ctx.data().db)
							.await?;
						// Delete old message
						old_message.delete(ctx).await?;
					} else {
						ctx.send(poise::CreateReply::default()
							.embed(
								templates::status::error(
									None,
									format!("Failed to migrate role list {:?}. Either the wrong channel is stored or the message doesn't exist.", entry.group_name)
								)
							)
							// .embed(templates::state_embed(false, &format!("Failed to migrate role list \"{}\". Either the wrong channel is stored or the message doesn't exist.", entry.name)))
						).await?;
					}
				}
			} else {
				ctx.send(poise::CreateReply::default()
					.embed(
						templates::status::warning(
							None,
							"Role lists were detected but no channel is stored for them. Existing role messages will still function, however the nature of this error means that they can't be deleted properly or migrated. In order to fix this make sure correct permissions are set on the channel where the old role lists are and then select that channel using this command."
						)
					)
					// .embed(templates::state_embed(false, "Role lists were detected but no channel is stored for them. Existing role messages will still function, however the nature of this error means that they can't be deleted properly or migrated. In order to fix this make sure correct permissions are set on the channel where the old role lists are and then select that channel using this command."))
				).await?;
			}
		}

		// Update the database with the new channel
		sqlx::query("UPDATE guild_settings SET roles_channel = ? WHERE guild_id = ?;")
			.bind(chid.get() as i64)
			.bind(ctx.guild_id().unwrap().get() as i64)
			.execute(&ctx.data().db)
			.await?;

		// Send confirm
		interaction.create_response(ctx, serenity::CreateInteractionResponse::UpdateMessage(serenity::CreateInteractionResponseMessage::new()
			.embed(serenity::CreateEmbed::new()
				.title("Role Settings Confirmed!")
				.color(colors::INFO)
				.description(format!("Roles will now be managed in {}.", chid.mention()))
			)
		)).await?;
	// If selected is none, that means the channel can't be posted to.
	} else {
		// interaction.create_response(ctx, serenity::CreateInteractionResponse::UpdateMessage(serenity::CreateInteractionResponseMessage::new()
			// .embed(templates::state_embed(false, "Channel is inaccessable for posting in. Either change the permission overrides or choose a different channel."))
		// )).await?;
		return Err(UserError(BotError::ChannelInaccessable.into()).into())
	}
	Ok(())
}
