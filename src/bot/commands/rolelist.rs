use std::time::Duration;
use poise::serenity_prelude as serenity;
use sqlx::{
	query,
	query_as
};
use abby_utils::{
	db_structs,
	concat,
	cmd_err
};
use crate::{Context, Error};

/// Creates a list of roles.
#[poise::command(
	guild_only,
	slash_command,
	required_permissions="MANAGE_ROLES",
	ephemeral
)]
pub async fn rolelist(
	ctx: Context<'_>,
	#[description = "Name of the role group."]
	group: String
) -> Result<(), Error> {
	let id = *ctx.guild_id().unwrap().as_u64();
	let srv_features = query_as::<_, db_structs::Server>("SELECT * FROM servers WHERE srvid = ?;")
		.bind(id as i64)
		.fetch_one(&ctx.data().db)
		.await?;
	if !srv_features.roles {
		abby_utils::feature_not_enabled(ctx).await?;
		return Ok(())
	}
	let role_sets = query_as::<_, db_structs::ServerRoles>("SELECT * FROM roles WHERE srvid = ?;")
		.bind(id as i64)
		.fetch_one(&ctx.data().db)
		.await?;

	let roles = abby_utils::all_roles_select(ctx, None).await?;
	let reply = ctx.send(|b| {
		b.content("Please choose roles from the list.")
			.components(|c| {
				c.create_action_row(|r| {
					r.create_select_menu(|menu| {
						menu.custom_id("rolelist.new");
						menu.placeholder("Select a set of roles.");
						menu.min_values(0);
						menu.max_values(roles.len() as u64);
						menu.options(|f| {
							f.set_options(roles)
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

	// let i = match reply.message().await?
	// 	.await_component_interaction(ctx)
	// 		.author_id(ctx.author().id)
	// 		.timeout(Duration::from_secs(300))
	// 		.await {
	// 			Some(inter_id) => {
	// 				inter_id.data.to_owned()
	// 			},
	// 			None => {
	// 				cmd_err("Interaction timed out, please try again.", "", ctx, reply).await?;
	// 				return Ok(())
	// 			}
	// 		};

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
		"rolelist.new" => {
			match &interaction {
				Some(m) => m.data.values.clone(),
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
	let mut insert_str: String = String::new();

	for id in &selected {
		insert_str = concat(&insert_str, &format!("({id}, \"{group}\", 0),"))
	}
	insert_str = insert_str.trim_end_matches(',').to_string();

	query(&format!("INSERT INTO {} (id, grp, users) VALUES{insert_str};", role_sets.tab))
		.execute(&ctx.data().db)
		.await?;
	let rolelist = query_as::<_, db_structs::RoleEntry>(&format!("SELECT * FROM {} WHERE grp = ?;", role_sets.tab))
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
		m.embed(|e|{
			e.title(&group);
			e.color(serenity::utils::Color::from_rgb(102, 51, 102));
			let mut name_str = String::new();
			let mut user_str = String::new();
			for r in rolelist {
				name_str = concat(&name_str, &format!("{}\n", serenity::RoleId(r.id as u64).to_role_cached(ctx).unwrap().name));
				user_str = concat(&user_str, &format!("{}\n", r.users));
			}
			name_str = name_str.trim_end_matches('\n').to_string();
			user_str = user_str.trim_end_matches('\n').to_string();
			e.field("Name", name_str, true);
			e.field("Users", user_str, true)
		});
		m.components(|c| {
			c.create_action_row(|row| {
				row.create_button(|button| {
					button.custom_id(concat("roles.pick.", group.as_str()));
					button.label("Pick");
					button.style(serenity::ButtonStyle::Primary)
				});
				row.create_button(|button| {
					button.custom_id(concat("roles.edit.", group.as_str()));
					button.label("Edit");
					button.style(serenity::ButtonStyle::Secondary)
				});
				row.create_button(|button| {
					button.custom_id(concat("roles.remove.", group.as_str()));
					button.label("Remove");
					button.style(serenity::ButtonStyle::Danger)
				})
			})
		})
	}).await?;
	Ok(())
}
