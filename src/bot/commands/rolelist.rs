use poise::serenity_prelude::{self as serenity, CacheHttp};
use sqlx::{
	query,
	query_as
};
use abby_utils::db_structs;
use crate::{Context, Error, templates, EMBED_STD};

/// Creates or deletes a list of roles to select from.
///
/// Using `setup roles` before using the create subcommend is *highly* recommended. Otherwise this command just dumps everything in the system channel.
#[poise::command(
	guild_only,
	slash_command,
	required_permissions="MANAGE_ROLES",
	category="Administration",
	ephemeral,
	subcommands("create", "delete")
)]
pub async fn rolelist(
	ctx: Context<'_>
) -> Result<(), Error> {
	ctx.say("You shouldn't be here?").await?;
	Ok(())
}

async fn autocomplete_groups<'a>(
	ctx: Context<'_>,
	partial: &'a str
) -> Vec<String> {
	let srv_id = ctx.guild_id().unwrap();
	let groups = query_as::<_, db_structs::RoleGroup>(&format!("SELECT * FROM rgroups_{srv_id};"))
		.fetch_all(&ctx.data().db)
		.await
		.unwrap();
	groups.into_iter()
		.filter_map(|f| {
			if f.name.starts_with(partial) {
				return Some(f.name)
			}
			None
		}).collect::<Vec<String>>()
}

#[poise::command(
	guild_only,
	slash_command,
	required_permissions="MANAGE_ROLES",
	category="Administration",
	ephemeral
)]
async fn delete(
	ctx: Context<'_>,
	#[description = "Name of the role group."]
	#[autocomplete = "autocomplete_groups"]
	group: String
) -> Result<(), Error> {
	let srv_id = ctx.guild_id().unwrap();
	let srv_features = query_as::<_, db_structs::Server>("SELECT * FROM servers WHERE srvid = ?;")
		.bind(*srv_id.as_u64() as i64)
		.fetch_one(&ctx.data().db)
		.await?;
	if !srv_features.roles {
		abby_utils::feature_not_enabled(ctx).await?;
		return Ok(())
	}
	// Info Gathering
	// let group_msg = query_as::<_, db_structs::RoleGroup>(&format!("SELECT * FROM rgroups_{srv_id} WHERE name = ?;"))
	// 	.bind(&group)
	// 	.fetch_optional(&ctx.data().db)
	// 	.await?;
	let group_msg = if let Some(rgroup) = query_as::<_, db_structs::RoleGroup>(&format!("SELECT * FROM rgroups_{srv_id} WHERE name = ?;"))
		.bind(&group)
		.fetch_optional(&ctx.data().db)
		.await? {
			rgroup
	} else {
		ctx.send(|m| {
			m.content("");
			m.ephemeral(true);
			m.embed(|e| {
				templates::builder_state_embed(e, false, &format!("Role group \"{group}\" does not exist."));
				e
			})
		}).await?;
		return Ok(())
	};
	let group_channel = query_as::<_, db_structs::ServerRoles>("SELECT * FROM role_options WHERE srvid = ?;")
		.bind(*srv_id.as_u64() as i64)
		.fetch_one(&ctx.data().db)
		.await?
		.channel
		.unwrap_or(*srv_id
			.to_guild_cached(ctx)
			.unwrap()
			.system_channel_id
			.unwrap()
			.as_u64() as i64
		);

	// Deletions
	ctx.http().delete_message(group_channel as u64, group_msg.msg as u64).await?;
	query(&format!("DELETE FROM rgroups_{srv_id} WHERE name = ?;"))
		.bind(&group)
		.execute(&ctx.data().db)
		.await?;
	query(&format!("DELETE FROM roles_{srv_id} WHERE grp = ?;"))
		.bind(&group)
		.execute(&ctx.data().db)
		.await?;

	// Respond
	ctx.send(|m| {
		m.content("");
		m.ephemeral(true);
		m.embed(|e| {
			templates::builder_state_embed(e, true, &format!("Role group \"{group}\" has been deleted."));
			e
		})
	}).await?;

	// Respond
	// mci.create_interaction_response(ctx, |i| {
	// 	i.kind(serenity::InteractionResponseType::ChannelMessageWithSource);
	// 	i.interaction_response_data(|m| {
	// 		m.content("");
	// 		m.ephemeral(true);
	// 		m.set_embed(templates::state_embed(true, &format!("Role group {group} has been deleted.")));
	// 		m.components(|c| c)
	// 	})
	// }).await?;
	Ok(())
}

/// Creates a list of roles.
#[poise::command(
	guild_only,
	slash_command,
	required_permissions="MANAGE_ROLES",
	category="Administration",
	ephemeral
)]
async fn create(
	ctx: Context<'_>,
	#[description = "Name of the role group."]
	group: String
) -> Result<(), Error> {
	let srv_id = *ctx.guild_id().unwrap().as_u64();
	let srv_features = query_as::<_, db_structs::Server>("SELECT * FROM servers WHERE srvid = ?;")
		.bind(srv_id as i64)
		.fetch_one(&ctx.data().db)
		.await?;
	if !srv_features.roles {
		abby_utils::feature_not_enabled(ctx).await?;
		return Ok(())
	}
	let role_sets = query_as::<_, db_structs::ServerRoles>("SELECT * FROM role_options WHERE srvid = ?;")
		.bind(srv_id as i64)
		.fetch_one(&ctx.data().db)
		.await?;

	// Select sorting
	let mut select: Vec<serenity::Role> = ctx.guild().unwrap().roles
		.into_values()
		.collect();
	select.sort_by(|a, b| {
		let a_l = a.name.to_lowercase();
		let b_l = b.name.to_lowercase();
		a_l.cmp(&b_l)
	});
	let select: Vec<serenity::CreateSelectMenuOption> = select.into_iter().filter_map(|r| {
		if r.name != "@everyone" {
			return Some(serenity::CreateSelectMenuOption::new(&r.name, r.id))
		}
		None
	}).collect();

	// The role selection menu.
	let reply = ctx.send(|b| {
		b.content("");
		b.embed(|e| {
			e.color(EMBED_STD);
			e.description("Please select a set of roles for the group below.")
		});
		b.components(|c| {
			c.create_action_row(|r| {
				r.create_select_menu(|menu| {
					menu.custom_id("rolelist.new");
					menu.placeholder("Roles");
					menu.min_values(1);
					menu.max_values(select.len() as u64);
					menu.options(|f| {
						f.set_options(select)
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
						// e.title(":warning: Failure :warning:");
						// e.color(EMBED_FAIL);
						// e.description("Interaction timed out, please try again.")
					})
				}).await?;

				return Ok(())
			}
		};

	// Processing message
	reply.edit(ctx, |b| {
		b.content("")
			.embed(|e| {
				templates::builder_processing_embed(e);
				e
				// e.color(EMBED_WAIT);
				// e.description("Processing, please wait...")
			})
			.components(|f| f)
	})
	.await?;

	// Insert roles into database.
	for rid in &interaction.data.values {
		query(&format!("INSERT INTO roles_{srv_id} (id, grp, users) VALUES(?, ?, 0);"))
			.bind(rid)
			.bind(&group)
			.execute(&ctx.data().db)
			.await?;
	}

	// Channel from new rolelist
	let rolelist = query_as::<_, db_structs::RoleEntry>(&format!("SELECT * FROM roles_{srv_id} WHERE grp = ?;"))
		.bind(&group)
		.fetch_all(&ctx.data().db)
		.await?;
	let channel = if let Some(c) = role_sets.channel {
		serenity::ChannelId::from(c as u64)
	} else {
		let default = ctx.guild().unwrap();
		default.default_channel(ctx.framework().bot_id).await.unwrap().id
	};

	// Generate list
	let msg = channel.send_message(ctx, |m| {
		m.content("");
		m.set_embed(templates::rolelist_embed(ctx.serenity_context(), &group, rolelist));
		m.set_components(templates::rolelist_components(&group))
	}).await?;

	// Add msg to group table
	query(&format!("INSERT INTO rgroups_{srv_id} (name, msg) VALUES (?, ?)"))
		.bind(&group)
		.bind(*msg.id.as_u64() as i64)
		.execute(&ctx.data().db)
		.await?;

	// Notify Success
	reply.edit(ctx, |m| {
		m.embed(|e| {
			templates::builder_state_embed(e, true, &format!("Rolelist for group {group} has been created."));
			e
			// e.title(":white_check_mark: Success :white_check_mark:");
			// e.color(EMBED_STD);
			// e.description(format!("Rolelist for group {group} has been created."))
		})
	}).await?;
	Ok(())
}
