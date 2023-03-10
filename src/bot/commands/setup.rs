use std::{
	time::Duration,
	collections::HashMap,
	str::FromStr
};
use poise::serenity_prelude as serenity;
use abby_utils::{
	Context,
	Error,
	cmd_err,
	concat,
	db_structs
};
use sqlx::{
	// FromRow,
	query,
	query_as
};

/// Administrates the bot on a per-server basis.
///
/// This command has various subcommands that aid in bot administration.
/// All subcommands are ephemeral, meaning they only show up for the person that invokes them.
/// They also only work for people with the correct permissions.
#[poise::command(
	guild_only,
	slash_command,
	required_permissions="ADMINISTRATOR",
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
	required_permissions="ADMINISTRATOR",
	ephemeral
)]
async fn bot(ctx: Context<'_>) -> Result<(), Error> {
	let id = *ctx.guild_id().unwrap().as_u64();

	let srv_features = query_as::<_, db_structs::Server>("SELECT * FROM servers WHERE srvid = ?;")
		.bind(id as i64)
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

	let interaction = reply
		.message()
		.await?
		.await_component_interaction(ctx)
			.author_id(ctx.author().id)
			.timeout(Duration::from_secs(300))
		.await;

	reply.edit(ctx, |b| {
		b.content("")
			.embed(|e| {
				e.color(serenity::utils::Color::from_rgb(253, 253, 150));
				e.description("Processing, please wait...")
			})
			.components(|f| f)
	})
	.await?;

	let interaction_id = match &interaction {
		Some(m) => &m.data.custom_id,
		None => {
			cmd_err("Interaction timed out, please try again.", "", ctx, reply).await?;
			return Ok(());
		}
	};

	let selected = match interaction_id.as_str() {
		"setup.bot" => {
			match &interaction {
				Some(m) => m.data.values.to_owned(),
				None => {
					cmd_err(":warning: Interaction Has No Data :warning:", "Interaction Has No Data", ctx, reply).await?;
					return Ok(());
				}
			}
		},
		o => {
			cmd_err(":warning: Unknown Interaction ID :warning:", format!("Unknown Interaction ID {o}").as_str(), ctx, reply).await?;
			return Ok(());
		}
	};

	let set_string = srv_features.as_array()
		.map(|(name, _)| {
			format!("{name} = {}", selected.contains(&concat("enable_", name)))
		}).join(",");
	query(&format!("UPDATE servers SET {set_string} WHERE srvid = {};", srv_features.srvid))
		.execute(&ctx.data().db)
		.await?;

	let updated = query_as::<_, db_structs::Server>("SELECT * FROM servers WHERE srvid = ?;")
		.bind(id as i64)
		.fetch_one(&ctx.data().db)
		.await?;
	reply.edit(ctx, |b| {
		b.content("")
			.embed(|e| {
				e.title("Feature Changes Confirmed!");
				e.color(serenity::utils::Color::from_rgb(102, 51, 102));
				e.description("Your new settings are:");
				for (name, value) in updated.as_array() {
					e.field(name[0..1].to_uppercase() + &name[1..], if value {"Enabled"} else {"Disabled"}, false);
				}
				e
			})
			.components(|f| f)
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
			query(&format!("CREATE TABLE IF NOT EXISTS roles_{id} (id BIGINT PRIMARY KEY NOT NULL, grp TEXT NOT NULL, users INTEGER NOT NULL);"))
				.execute(&ctx.data().db)
				.await?;
			query("INSERT INTO role_options (srvid) VALUES(?);")
				.bind(updated.srvid)
				.execute(&ctx.data().db)
				.await?;
			ctx.send(|b| {
				b.content("")
					.embed(|e| {
						e.description("Note that using the default channel for roles is not advised. It is recommended to run `/setup roles` now.");
						e.color(serenity::utils::Color::from_rgb(250,128,114))
					})
			}).await?;
		} else {
			query(&format!("DROP TABLE IF EXISTS roles_{id};"))
				.execute(&ctx.data().db)
				.await?;
			query("DELETE FROM role_options WHERE srvid = ?;")
				.bind(updated.srvid)
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
	required_permissions="ADMINISTRATOR",
	ephemeral
)]
async fn roles(ctx: Context<'_>) -> Result<(), Error> {
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

	let interaction = reply
		.message()
		.await?
		.await_component_interaction(ctx)
			.author_id(ctx.author().id)
			.timeout(Duration::from_secs(300))
		.await;

	reply.edit(ctx, |b| {
		b.content("")
			.embed(|e| {
				e.color(serenity::utils::Color::from_rgb(253, 253, 150));
				e.description("Processing, please wait...")
			})
			.components(|f| f)
	})
	.await?;

	let interaction_id = match &interaction {
		Some(m) => &m.data.custom_id,
		None => {
			cmd_err("Interaction timed out, please try again.", "", ctx, reply).await?;
			return Ok(());
		}
	};

	let selected = match interaction_id.as_str() {
		"setup.roles" => {
			match &interaction {
				Some(m) => m.data.values.to_owned(),
				None => {
					cmd_err(":warning: Interaction Has No Data :warning:", "Interaction Has No Data", ctx, reply).await?;
					return Ok(());
				}
			}
		},
		o => {
			cmd_err(":warning: Unknown Interaction ID :warning:", format!("Unknown Interaction ID {o}").as_str(), ctx, reply).await?;
			return Ok(());
		}
	};

	let selid = if let Some(s) = selected.first() {
		s.as_str()
	} else {"0"};
	let chid = if selid != "0" {
		Some(selid.parse::<i64>()?)
		// Some(*serenity::ChannelId::from_str(selid)?.as_u64() as i64)
	} else {None};

	query("UPDATE role_options SET channel = ? WHERE srvid = ?;")
		.bind(chid)
		.bind(*ctx.guild_id().unwrap().as_u64() as i64)
		.execute(&ctx.data().db)
		.await?;

	let chname = if let Some(x) = ctx.guild().unwrap().channels.get(&serenity::ChannelId::from_str(selid)?) {
		concat("#", x.clone().guild().unwrap().name.as_str())
	} else {"default announcements".to_string()};

	reply.edit(ctx, |b| {
		b.content("")
			.embed(|e| {
				e.title("Role Settings Confirmed!");
				e.color(serenity::utils::Color::from_rgb(102, 51, 102));
				e.description(format!("Roles will now be managed in the {chname} channel."))
			})
			.components(|f| f)
	})
	.await?;
	Ok(())
}
