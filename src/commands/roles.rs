use crate::{Context, Error};
use std::{
	time::Duration, collections::HashMap,
	// str::FromStr
};
use poise::serenity_prelude as serenity;
use time;

async fn fetch_all_roles(ctx: Context<'_>) -> Result<Vec<serenity::builder::CreateSelectMenuOption>, Error>{
	let hash = ctx.guild()
		.expect("Could not fetch guild from cache.")
		.roles;
	let mut list: Vec<serenity::builder::CreateSelectMenuOption> = Vec::new();
	for (rid, r) in hash {
		if r.name != "@everyone" {
			// let h = if ctx.author().has_role(&ctx, ctx.guild_id().unwrap(), &r).await? { true } else { false };
			list.push(serenity::builder::CreateSelectMenuOption::new(r.name, rid)
				// .default_selection(h)
				.to_owned())
		}
	};
	Ok(list)
}

/// Role Test
#[poise::command(
	guild_only,
	slash_command,
	required_permissions="MANAGE_ROLES",
	category="Administration",
	ephemeral
)]
pub async fn roles(
	ctx:Context<'_>
) -> Result<(), Error> {
	let roles = fetch_all_roles(ctx).await?;

	let reply = ctx.send(|b| {
		b.content("Please choose roles from the list.")
			.components(|c| {
				c.create_action_row(|row| {
					row.create_select_menu(|menu| {
						menu.custom_id("rolelist.all");
						menu.placeholder("Select a role");
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

	let interaction_id = match &interaction {
		Some(m) => &m.data.custom_id,
		None => {
			reply.edit(ctx, |b| {
					b.components(|b| b).content("Interaction timed out, please try again.")
				})
				.await?;
			return Ok(());
		}
	};

	let selected = match interaction_id.as_str() {
		"rolelist.all" => {
			match &interaction {
				Some(m) => m.data.values.clone(),
				None => {
					reply.edit(ctx, |b| {
							b.components(|b| b).content(":warning: Interaction Has No Data :warning:")
						})
						.await?;
					eprintln!("Interaction Has No Data in \"{}\"", ctx.guild().unwrap().name);
					return Ok(());
				}
			}
		},
		other => {
			reply.edit(ctx, |b| {
					b.components(|b| b).content(":warning: Unknown Interaction ID :warning:")
				})
				.await?;
			eprintln!("{} - Unknown Interaction ID in \"{}\": {:?}", time::OffsetDateTime::now_utc()
					.to_offset(time::UtcOffset::current_local_offset()?)
					.format(&time::format_description::parse("[year]-[month]-[day]T[hour]:[minute]:[second][offset_hour sign:mandatory]:[offset_minute]")?)?,
				ctx.guild().unwrap().name, other);
			return Ok(());
		}
	};

	// reply.edit(ctx, |b| {
	// 		b.components(|b| b).content(format!("{:#?}", selected))
	// 	})
	// 	.await?;
	// ctx.data().rlist

	// let mut mem = ctx.author_member()
	// 	.await
	// 	.expect("Could not get Member data")
	// 	.into_owned();

	// for role in selected {
	// 	let rid = serenity::RoleId::from_str(role.as_str())?;
	// 	mem.add_role(ctx, rid).await?
	// }

	let mut new: HashMap<String, String> = HashMap::new();

	for rid in selected {
		let name = ctx.guild().unwrap().roles.iter()
			.find_map(|(key, val)| if key.to_string() == rid { Some(val.name.to_owned()) } else { None });
		new.insert(rid, name.unwrap());
	};

	reply.delete(ctx).await?;
	// interaction.unwrap().delete_original_interaction_response(ctx).await?;

	// let secondary = ctx.channel_id().send_message(ctx, |b| {
	// 	b.content("Please choose roles from the list.")
	// 		.components(|c| {
	// 			c.create_action_row(|row| {
	// 				for (rid, name) in new {
	// 					row.create_button(|button| {
	// 						button.custom_id(format!("roleadd.{}", rid));
	// 						button.label(name);
	// 						button.style(serenity::ButtonStyle::Primary)
	// 					});
	// 				}
	// 				row
	// 			})
	// 		})
	// }).await?;

	// let secondary_inter = secondary
	// 	.await_component_interaction(ctx)
	// 	.await;

	// let secondary_id = match &secondary_inter {
	// 	Some(m) => &m.data.custom_id,
	// 	None => {
	// 		reply.edit(ctx, |b| {
	// 				b.components(|b| b).content("Interaction timed out, please try again.")
	// 			})
	// 			.await?;
	// 		return Ok(());
	// 	}
	// };

	// ctx.channel_id().say(ctx, format!("{}",secondary_id)).await?;

	ctx.channel_id().send_message(ctx, |b| {
		b.content("Please choose roles from the list.")
			.components(|c| {
				c.create_action_row(|row| {
					for (rid, name) in new {
						row.create_button(|button| {
							button.custom_id(format!("roleadd.{}", rid));
							button.label(name);
							button.style(serenity::ButtonStyle::Primary)
						});
					}
					row
				})
			})
	}).await?;

	Ok(())
}
