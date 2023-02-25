use std::{
	time::Duration,
	// collections::HashMap
};
use poise::serenity_prelude as serenity;
use abby_utils::{Context, Error, cmd_err};

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
	subcommands("rolelist", "bot")
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
	ctx.say("Test!").await?;
	Ok(())
}

/// Creates a new role list.
#[poise::command(
	guild_only,
	slash_command,
	required_permissions="MANAGE_ROLES",
	ephemeral
)]
async fn rolelist(ctx: Context<'_>) -> Result<(), Error> {
	let roles = abby_utils::all_roles_select(ctx, None).await?;

	let reply = ctx.send(|b| {
		b.content("Please choose roles from the list.")
			.components(|c| {
				c.create_action_row(|row| {
					row.create_select_menu(|menu| {
						menu.custom_id("rolelist.create");
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

	let interaction_id = match &interaction {
		Some(m) => &m.data.custom_id,
		None => {
			cmd_err("Interaction timed out, please try again.", "", ctx, reply).await?;
			return Ok(());
		}
	};

	let selected = match interaction_id.as_str() {
		"rolelist.create" => {
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

	reply.delete(ctx).await?;

	let new = abby_utils::roles_from_selected(selected, ctx.guild().unwrap().roles);
	ctx.channel_id().send_message(ctx, |b| {
		b.content("Please choose roles from the list.")
			.components(|c| {
				c.create_action_row(|row| {
					for (rid, r) in new {
						row.create_button(|button| {
							button.custom_id(format!("roleadd.{rid}"));
							button.label(r.name);
							button.style(serenity::ButtonStyle::Primary)
						});
					}
					row
				});
				c.create_action_row(|row| {
					row.create_button(|button| {
						button.custom_id("edit.roles");
						button.label("Edit Roles");
						button.style(serenity::ButtonStyle::Secondary)
					})
				})
			})
	}).await?;

	Ok(())
}

