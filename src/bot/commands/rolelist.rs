use poise::serenity_prelude as serenity;
use sqlx::{
	query,
	query_as
};
use abby_utils::db_structs;
use crate::{Context, Error, templates, EMBED_FAIL, EMBED_WAIT, EMBED_STD};

/// Creates a list of roles.
#[poise::command(
	guild_only,
	slash_command,
	required_permissions="MANAGE_ROLES",
	category="Administration",
	ephemeral
)]
pub async fn rolelist(
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
		.into_iter().map(|(_, r)| r).collect();
	select.sort_by(|a, b| {
		let a_l = a.name.to_lowercase();
		let b_l = b.name.to_lowercase();
		return a_l.cmp(&b_l)
	});
	let select: Vec<serenity::CreateSelectMenuOption> = select.into_iter().filter_map(|r| {
		if r.name != "@everyone" {
			return Some(serenity::CreateSelectMenuOption::new(&r.name, r.id)
			.to_owned())
		}
		None
	}).collect();

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
						e.title(":warning: Failure :warning:");
						e.color(EMBED_FAIL);
						e.description("Interaction timed out, please try again.")
					})
				}).await?;

				return Ok(())
			}
		};
	reply.edit(ctx, |b| {
		b.content("")
			.embed(|e| {
				e.color(EMBED_WAIT);
				e.description("Processing, please wait...")
			})
			.components(|f| f)
	})
	.await?;

	for rid in &interaction.data.values {
		query(&format!("INSERT INTO roles_{srv_id} (id, grp, users) VALUES(?, ?, 0);"))
			.bind(rid)
			.bind(&group)
			.execute(&ctx.data().db)
			.await?;
	}
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
	channel.send_message(ctx, |m| {
		m.content("");
		m.set_embed(templates::rolelist_embed(ctx.serenity_context(), &group, rolelist));
		m.set_components(templates::rolelist_components(&group))
	}).await?;
	reply.edit(ctx, |m| {
		m.embed(|e| {
			e.title(":white_check_mark: Success :white_check_mark:");
			e.color(EMBED_STD);
			e.description(format!("Rolelist for group {group} has been created."))
		})
	}).await?;
	Ok(())
}
