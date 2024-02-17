use poise::serenity_prelude::{self as serenity, Mentionable};
use sqlx::{
	query,
	query_as
};
use crate::{
	structs::db,
	templates, utils,
	Context, Error,
	EMBED_STD, EMBED_WAIT, EMBED_FAIL
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
pub async fn setup(ctx: Context<'_>) -> Result<(), Error> {
	ctx.say("You shouldn't be here?").await?;
	Ok(())
}

/// Sets up various settings for the bot on the server.
#[poise::command(
	guild_only,
	slash_command,
	required_permissions="MANAGE_GUILD",
	ephemeral
)]
async fn bot(ctx: Context<'_>) -> Result<(), Error> {
	let srv_id = ctx.guild_id().unwrap().get();
	let guild = ctx.guild().unwrap().clone();

	let srv_features = query_as::<_, db::Server>("SELECT * FROM servers WHERE srvid = ?;")
		.bind(srv_id as i64)
		.fetch_one(&ctx.data().db)
		.await
		.unwrap();

	let reply = ctx.send(poise::CreateReply::default()
		.embed(serenity::CreateEmbed::new()
			.title("Bot Configuration")
			.color(EMBED_STD)
			.description("Below is a select menu to choose my features on a per-server basis. These can be changed at any time by simply running the command again.")
			.field("Serious", "Toggles the appearence of memes, injokes or other related content. This setting encompasses *all* invocations of this across all other features.", false)
			.field("Messages", "Whether or not I should listen to message events outside of command invocations. This is mostly to reply to them based on regex.", false)
			.field("Roles", "Toggles role management features. Note that the subcommand is global so while it will still exist, it will simply do nothing.", false)
		).components(vec![
			serenity::CreateActionRow::SelectMenu(serenity::CreateSelectMenu::new("setup.bot", serenity::CreateSelectMenuKind::String { options: srv_features.as_selectmenuoptions() })
				.placeholder("Please select features...")
				.min_values(0)
				.max_values(srv_features.as_selectmenuoptions().len() as u8))
		])
	).await?;

	let interaction = match reply.message().await?
		.await_component_interaction(ctx)
		.author_id(ctx.author().id)
		.timeout(std::time::Duration::from_secs(300))
		.await {
		Some(i) => i,
		None => {
			reply.edit(ctx, poise::CreateReply::default()
				.embed(templates::state_embed(false, "Interaction timed out, please try again."))
				.components(Vec::new())
			).await?;
			return Ok(())
		}
	};

	reply.edit(ctx, poise::CreateReply::default()
		.embed(templates::processing_embed())
		.components(Vec::new())
	).await?;

	let selected = match &interaction.data.kind {
		serenity::ComponentInteractionDataKind::StringSelect { values } => values,
		_ => {
			interaction.create_response(ctx, serenity::CreateInteractionResponse::UpdateMessage(serenity::CreateInteractionResponseMessage::new()
				.embed(templates::state_embed(false, "Somehow recieved wrong interaction, please report this."))
				.components(Vec::new())
			)).await?;
			return Ok(());
		}
	};

	let set_string = srv_features.as_array()
		.map(|(name, _)| format!("{name} = {}", selected.contains(&utils::concat("enable_", name))))
		.join(",");
	query(&format!("UPDATE servers SET {set_string} WHERE srvid = {};", srv_features.srvid))
		.execute(&ctx.data().db)
		.await
		.unwrap();

	let updated = query_as::<_, db::Server>("SELECT * FROM servers WHERE srvid = ?;")
		.bind(srv_id as i64)
		.fetch_one(&ctx.data().db)
		.await
		.unwrap();
	interaction.create_response(ctx, serenity::CreateInteractionResponse::UpdateMessage(serenity::CreateInteractionResponseMessage::new()
		.embed(serenity::CreateEmbed::new()
			.title("Feature Changes Confirmed!")
			.color(EMBED_STD)
			.description("Your new settings are:")
			.fields(updated.as_array()
				.map(|(name, value)|  (name[0..1].to_uppercase() + &name[1..], if value {"Enabled"} else {"Disabled"}, false))
			)
		)
	)).await?;
	// Changed Serious
	// let gid = serenity::GuildId(updated.srvid as u64);
	// if updated.serious != srv_features.serious {
		// use super::bottomify;
		// let cmds = poise::builtins::create_application_commands(&vec![
		// 	bottomify::bottomify()
		// ]);
		// if updated.serious {
		// 	// gid.set_application_commands(ctx, |c| {
		// 	// 	*c = cmds;
		// 	// 	c
		// 	// }).await?;
		// } else {
		// 	// gid.set_application_commands(ctx, |c| c).await?;
		// }
	// }
	// Changed Roles
	if updated.roles != srv_features.roles {
		if updated.roles {
			query(&format!("CREATE TABLE IF NOT EXISTS roles_{srv_id} (id BIGINT PRIMARY KEY NOT NULL, grp TEXT NOT NULL, users INTEGER NOT NULL);"))
				.execute(&ctx.data().db)
				.await
				.unwrap();
			query(&format!("CREATE TABLE IF NOT EXISTS rgroups_{srv_id} (name TEXT PRIMARY KEY NOT NULL, msg BIGINT NOT NULL);"))
				.execute(&ctx.data().db)
				.await
				.unwrap();
			// Find default channel in i64 form (for database)
			let default = utils::default_bot_channel(ctx, guild, ctx.framework().bot_id).await;
			// Insert into role_options
			query("INSERT INTO role_options (srvid, channel) VALUES(?, ?);")
				.bind(updated.srvid)
				.bind(default.map(|c| c.get() as i64))
				.execute(&ctx.data().db)
				.await?;
			// Send message
			ctx.send(poise::CreateReply::default()
				.embed(
					if let Some(channel) = default {
						// Default channel exists
						serenity::CreateEmbed::new()
							.description(format!("The current default channel for role lists is {}. If this is not wanted, please run `/setup roles` now.", channel.mention()))
							.color(EMBED_WAIT)
					} else {
						// Default channel does not exist
						serenity::CreateEmbed::new()
							.description("Could not setup a default channel for roles. Before creating any lists running `/setup roles` is needed.")
							.color(EMBED_FAIL)
					}
				)
			).await?;
		} else {
			// Delete messages
			let group_channel = query_as::<_, db::ServerRoles>("SELECT * FROM role_options WHERE srvid = ?;")
				.bind(srv_id as i64)
				.fetch_one(&ctx.data().db)
				.await
				.unwrap()
				.channel
				.unwrap_or(ctx.guild_id()
					.unwrap()
					.to_guild_cached(&ctx)
					.unwrap()
					.system_channel_id
					.unwrap_or(ctx.guild()
						.unwrap()
						.default_channel(ctx.framework().bot_id)
						.unwrap()
						.id)
					.get() as i64
				);
			for grp in query_as::<_, db::RoleGroup>(&format!("SELECT * FROM rgroups_{srv_id}"))
				.fetch_all(&ctx.data().db)
				.await? {
				if ctx.http().delete_message(serenity::ChannelId::new(group_channel as u64), serenity::MessageId::new(grp.msg as u64), Some("Role feature disabled, deleting groups.")).await.is_err() {
					ctx.send(poise::CreateReply::default()
						.ephemeral(true)
						.embed(templates::state_embed(false, &format!("Either can't find or can't delete the message for the role group \"{}\". Entries will be removed from the database, but the message will need to be deleted manually.", grp.name)))
					).await?;
				}
			}

			// Purge Database
			query(&format!("DROP TABLE IF EXISTS roles_{srv_id};"))
				.execute(&ctx.data().db)
				.await
				.unwrap();
			query("DELETE FROM role_options WHERE srvid = ?;")
				.bind(updated.srvid)
				.execute(&ctx.data().db)
				.await
				.unwrap();
			query(&format!("DROP TABLE IF EXISTS rgroups_{srv_id};"))
				.execute(&ctx.data().db)
				.await
				.unwrap();
		}
	}
	Ok(())
}

/// Edits settings for the role management feature.
#[poise::command(
	guild_only,
	slash_command,
	required_permissions="MANAGE_GUILD",
	ephemeral
)]
async fn roles(ctx: Context<'_>) -> Result<(), Error> {
	let guild = ctx.guild().unwrap().clone();

	// Data gathering
	let srv_features = query_as::<_, db::Server>("SELECT * FROM servers WHERE srvid = ?;")
		.bind(ctx.guild_id().unwrap().get() as i64)
		.fetch_one(&ctx.data().db)
		.await
		.unwrap();
	if !srv_features.roles {
		utils::feature_not_enabled(ctx).await?;
		return Ok(())
	}
	let role_opts = query_as::<_, db::ServerRoles>("SELECT * FROM role_options WHERE srvid = ?;")
		.bind(ctx.guild_id().unwrap().get() as i64)
		.fetch_optional(&ctx.data().db)
		.await
		.unwrap()
		.unwrap_or_default();

	// Channel select
	let reply = ctx.send(poise::CreateReply::default()
		.embed(serenity::CreateEmbed::new()
			.title("Role Setup")
			.color(EMBED_STD)
			.description("Please select a channel for role management to take place in. This channel should be completely empty save for the role lists. All interactions done with me via this channel will be ephemeral, so there should end up being no clutter.")
		).components(vec![
			serenity::CreateActionRow::SelectMenu(serenity::CreateSelectMenu::new("setup.roles", serenity::CreateSelectMenuKind::Channel { channel_types: Some(vec![serenity::ChannelType::Text]), default_channels: role_opts.channel
					.map(|c| vec![serenity::ChannelId::new(c as u64)])
				}).placeholder("Select a Channel.")
				.min_values(0)
				.max_values(1)
			)
		])
	).await?;

	// Await interaction
	let interaction = match reply.message().await?
		.await_component_interaction(ctx)
		.author_id(ctx.author().id)
		.timeout(std::time::Duration::from_secs(300))
		.await {
			Some(i) => i,
			None => {
				reply.edit(ctx, poise::CreateReply::default()
					.embed(templates::state_embed(false, "Interaction timed out, please try again."))
					.components(Vec::new())
				).await?;
				return Ok(())
			}
		};

	// Processing message
	reply.edit(ctx, poise::CreateReply::default()
		.embed(templates::processing_embed())
		.components(Vec::new())
	).await?;

	// Return either selected or default channel
	let selected = match &interaction.data.kind {
		serenity::ComponentInteractionDataKind::ChannelSelect { values } => {
			if let Some(ch) = values.first() {
				if utils::can_post(ctx, ch, ctx.framework().bot_id).await { Some(*ch) }
				else { None }
			} else {
				utils::default_bot_channel(ctx, guild, ctx.framework().bot_id).await
			}
		}
		_ => {
			interaction.create_response(ctx, serenity::CreateInteractionResponse::UpdateMessage(serenity::CreateInteractionResponseMessage::new()
				.embed(templates::state_embed(false, "Somehow recieved wrong interaction, please report this."))
				.components(Vec::new())
			)).await?;
			return Ok(());
		}
	};

	if let Some(chid) = selected {
		// Fetch old group messages
		let old_lists = query_as::<_, db::RoleGroup>(&format!("SELECT * FROM rgroups_{};", ctx.guild_id().unwrap().get()))
			.fetch_all(&ctx.data().db)
			.await?;

		// Migrates if there are role messages in the old channel
		if !old_lists.is_empty() {
			if let Some(c) = query_as::<_, db::ServerRoles>("SELECT * FROM role_options WHERE srvid = ?;")
				.bind(ctx.guild_id().unwrap().get() as i64)
				.fetch_one(&ctx.data().db)
				.await
				.unwrap()
				.channel {
				for entry in old_lists {
					// If the old message can't even be reached then no point trying anyway.
					if let Ok(old_message) = ctx.http().get_message(serenity::ChannelId::new(c as u64), serenity::MessageId::new(entry.msg as u64)).await {
						let new_message = chid.send_message(ctx, serenity::CreateMessage::new()
							// Copy embed
							.embed(serenity::CreateEmbed::from(old_message.embeds.first().unwrap().to_owned()))
							// Add components
							.components(vec![
								templates::rolelist_components(&entry.name)
							])
						).await?;
						// Update database with new message
						query(&format!("UPDATE rgroups_{} SET msg = ? WHERE name = ?;", ctx.guild_id().unwrap().get()))
							.bind(new_message.id.get() as i64)
							.bind(entry.name)
							.execute(&ctx.data().db)
							.await?;
						// Delete old message
						old_message.delete(ctx).await?;
					} else {
						ctx.send(poise::CreateReply::default()
							.embed(templates::state_embed(false, &format!("Failed to migrate role list \"{}\". Either the wrong channel is stored or the message doesn't exist.", entry.name)))
						).await?;
					}
				}
			} else {
				ctx.send(poise::CreateReply::default()
					.embed(templates::state_embed(false, "Role lists were detected but no channel is stored for them. Existing role messages will still function, however the nature of this error means that they can't be deleted properly or migrated. In order to fix this make sure correct permissions are set on the channel where the old role lists are and then select that channel using this command."))
				).await?;
			}
		}

		// Update the database with the new channel
		query("UPDATE role_options SET channel = ? WHERE srvid = ?;")
			.bind(chid.get() as i64)
			.bind(ctx.guild_id().unwrap().get() as i64)
			.execute(&ctx.data().db)
			.await?;

		// Send confirm
		interaction.create_response(ctx, serenity::CreateInteractionResponse::UpdateMessage(serenity::CreateInteractionResponseMessage::new()
			.embed(serenity::CreateEmbed::new()
				.title("Role Settings Confirmed!")
				.color(EMBED_STD)
				.description(format!("Roles will now be managed in {}.", chid.mention()))
			)
		)).await?;
	// If selected is none, that means the channel can't be posted to.
	} else {
		interaction.create_response(ctx, serenity::CreateInteractionResponse::UpdateMessage(serenity::CreateInteractionResponseMessage::new()
			.embed(templates::state_embed(false, "Channel is inaccessable for posting in. Either change the permission overrides or choose a different channel."))
		)).await?;
	}
	Ok(())
}
