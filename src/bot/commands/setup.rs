use std::{
	collections::HashMap
};
use poise::serenity_prelude::{self as serenity, CacheHttp, cache::FromStrAndCache};
use abby_utils::{
	Context,
	Error,
	concat,
	db_structs
};
use sqlx::{
	query,
	query_as
};

use crate::{EMBED_WAIT, EMBED_STD, templates};

/// Administrates the bot on a per-server basis.
///
/// ```Subcommands:
///  bot      Manages serverwide bot features.
///  roles    Manages the settings of the roles feature.
/// ```
/// All subcommands are ephemeral and require the `ADMINISTRATOR` permission.
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
	let srv_id = *ctx.guild_id().unwrap().as_u64();

	let srv_features = query_as::<_, db_structs::Server>("SELECT * FROM servers WHERE srvid = ?;")
		.bind(srv_id as i64)
		.fetch_one(&ctx.data().db)
		.await?;

	let reply = ctx.send(|b| {
		b.content("")
			.embed(|e| {
				e.title("Bot Configuration");
				e.color(serenity::utils::Color::from_rgb(102, 51, 102));
				e.description("Below is a select menu to setup my features on a per-server basis. These can be changed at any time simply by running the command again.");
				e.field("Serious", "Toggles the appearence of memes, injokes or other related content. This setting encompasses *all* invocations of this across all other features.", false);
				e.field("Messages", "Whether or not I should listen to message events outside of command invocations. This is mostly to reply to them based on regex.", false);
				e.field("Roles", "Toggles role management features. Note that the subcommand is global so while it will still exist, it will simply do nothing.", false)
			})
			.components(|c| {
				c.create_action_row(|row| {
					row.create_select_menu(|menu| {
						menu.custom_id("setup.bot");
						menu.placeholder("Please select features...");
						menu.min_values(0);
						menu.max_values(3);
						menu.options(|f| {
							f.set_options(srv_features.as_selectmenuoptions())
						})
					})
				})
			})
	}).await?;

	let interaction = match reply.message().await?
	.await_component_interaction(ctx)
		.author_id(ctx.author().id)
		.timeout(std::time::Duration::from_secs(300))
		.await {
			Some(i) => {
				i
			},
			None => {
				reply.edit(ctx, |f| {
					f.embed(|e| {
						templates::builder_state_embed(e, false, "Interaction timed out, please try again.");
						e
					})
				}).await?;

				return Ok(())
			}
		};

	reply.edit(ctx, |m| {
		m.content("");
		m.embed(|e| {
			templates::builder_processing_embed(e);
			e
		});
		m.components(|f| f)
	})
	.await?;

	let set_string = srv_features.as_array()
		.map(|(name, _)| {
			format!("{name} = {}", interaction.data.values.contains(&concat("enable_", name)))
		}).join(",");
	query(&format!("UPDATE servers SET {set_string} WHERE srvid = {};", srv_features.srvid))
		.execute(&ctx.data().db)
		.await?;

	let updated = query_as::<_, db_structs::Server>("SELECT * FROM servers WHERE srvid = ?;")
		.bind(srv_id as i64)
		.fetch_one(&ctx.data().db)
		.await?;
	reply.edit(ctx, |m| {
		m.content("");
		m.embed(|e| {
			e.title("Feature Changes Confirmed!");
			e.color(EMBED_STD);
			e.description("Your new settings are:");
			for (name, value) in updated.as_array() {
				e.field(name[0..1].to_uppercase() + &name[1..], if value {"Enabled"} else {"Disabled"}, false);
			}
			e
		});
		m.components(|f| f)
	})
	.await?;
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
				.await?;
			query(&format!("CREATE TABLE IF NOT EXISTS rgroups_{srv_id} (name TEXT PRIMARY KEY NOT NULL, msg BIGINT NOT NULL);"))
				.execute(&ctx.data().db)
				.await?;
			query("INSERT INTO role_options (srvid) VALUES(?);")
				.bind(updated.srvid)
				.execute(&ctx.data().db)
				.await?;
			ctx.send(|m| {
				m.content("");
				m.embed(|e| {
					e.description("Note that using the default channel for roles is not advised. It is recommended to run `/setup roles` now.");
					e.color(EMBED_WAIT)
				})
			}).await?;
		} else {
			// Delete messages
			let group_channel = query_as::<_, db_structs::ServerRoles>("SELECT * FROM role_options WHERE srvid = ?;")
				.bind(srv_id as i64)
				.fetch_one(&ctx.data().db)
				.await?
				.channel
				.unwrap_or(*ctx.guild_id()
					.unwrap()
					.to_guild_cached(ctx)
					.unwrap()
					.system_channel_id
					.unwrap()
					.as_u64() as i64
				);
			for grp in query_as::<_, db_structs::RoleGroup>(&format!("SELECT * FROM rgroups_{srv_id}"))
				.fetch_all(&ctx.data().db)
				.await? {
				ctx.http().delete_message(group_channel as u64, grp.msg as u64).await?;
			}

			// Purge Database
			query(&format!("DROP TABLE IF EXISTS roles_{srv_id};"))
				.execute(&ctx.data().db)
				.await?;
			query("DELETE FROM role_options WHERE srvid = ?;")
				.bind(updated.srvid)
				.execute(&ctx.data().db)
				.await?;
			query(&format!("DROP TABLE IF EXISTS rgroups_{srv_id};"))
				.execute(&ctx.data().db)
				.await?;
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
	// Data gathering
	let srv_features = query_as::<_, db_structs::Server>("SELECT * FROM servers WHERE srvid = ?;")
		.bind(*ctx.guild_id().unwrap().as_u64() as i64)
		.fetch_one(&ctx.data().db)
		.await?;
	if !srv_features.roles {
		abby_utils::feature_not_enabled(ctx).await?;
		return Ok(())
	}
	let role_opts = query_as::<_, db_structs::ServerRoles>("SELECT * FROM role_options WHERE srvid = ?;")
		.bind(*ctx.guild_id().unwrap().as_u64() as i64)
		.fetch_optional(&ctx.data().db)
		.await?
		.unwrap_or_default();
	let channels = ctx.guild().unwrap().channels.into_iter().filter(|(_, c)| {
		if let serenity::Channel::Guild(gc) = c {
			if let serenity::ChannelType::Text = gc.kind {
				return true
			}
		}
		false
	}).collect::<HashMap<serenity::ChannelId, serenity::Channel>>();
	let mut selectopts: Vec<serenity::CreateSelectMenuOption> = Vec::new();
	for (cid, c) in channels {
		selectopts.push(serenity::CreateSelectMenuOption::new(c.guild().unwrap().name, cid)
			.default_selection(if let Some(rch) = role_opts.channel {
				*cid.as_u64() as i64 == rch
			} else {false})
			.to_owned())
	}

	// Channel select
	let reply = ctx.send(|b| {
		b.content("")
			.embed(|e| {
				e.title("Role Setup");
				e.color(serenity::utils::Color::from_rgb(102, 51, 102));
				e.description("Please select a channel for role management to take place in. This channel should be completely empty save for the role lists. All interactions done with me via this channel will be ephemeral, so there should end up being no clutter.")
			})
			.components(|c| {
				c.create_action_row(|row| {
					row.create_select_menu(|menu| {
						menu.custom_id("setup.roles");
						menu.placeholder("Select a Channel.");
						menu.min_values(0);
						menu.max_values(1);
						menu.options(|f| {
							f.set_options(selectopts)
						})
					})
				})
			})
	}).await?;

	// Await interaction
	let interaction = match reply.message().await?
	.await_component_interaction(ctx)
		.author_id(ctx.author().id)
		.timeout(std::time::Duration::from_secs(300))
		.await {
			Some(i) => {
				i
			},
			None => {
				reply.edit(ctx, |f| {
					f.embed(|e| {
						templates::builder_state_embed(e, false, "Interaction timed out, please try again.");
						e
					})
				}).await?;

				return Ok(())
			}
		};

	// Processing message
	reply.edit(ctx, |b| {
		b.content("");
		b.embed(|e| {
			templates::builder_processing_embed(e);
			e
		}).components(|f| f)
	})
	.await?;

	// Return either selected
	let selected = if let Some(s) = interaction.data.values.first() {
		let id = serenity::ChannelId::from_str(ctx, s)?;
		if abby_utils::can_post(ctx, id, ctx.framework().bot_id).await {
			Some(id)
		} else {
			None
		}
	} else {
		abby_utils::default_bot_channel(ctx, ctx.guild().unwrap(), ctx.framework().bot_id).await
	};

	if let Some(chid) = selected {
		// Update the database with the new channel
		query("UPDATE role_options SET channel = ? WHERE srvid = ?;")
			.bind(*chid.as_u64() as i64)
			.bind(*ctx.guild_id().unwrap().as_u64() as i64)
			.execute(&ctx.data().db)
			.await?;

		// Get channel name from the id
		let chname = format!("#{}", ctx.guild().unwrap().channels.get(&chid).unwrap().clone().guild().unwrap().name());

		// Send confirm
		reply.edit(ctx, |b| {
			b.content("");
			b.embed(|e| {
				e.title("Role Settings Confirmed!");
				e.color(serenity::utils::Color::from_rgb(102, 51, 102));
				e.description(format!("Roles will now be managed in the {chname} channel."))
			});
			b.components(|f| f)
		})
		.await?;
	// If selected is none, that means the channel can't be posted to.
	} else {
		reply.edit(ctx, |b| {
			b.content("");
			b.embed(|e| {
				templates::builder_state_embed(e, false, "Channel is inaccessable for posting in. Either change the permission overrides or choose a different channel.");
				e
			});
			b.components(|f| f)
		}).await?;
	}
	Ok(())
}
