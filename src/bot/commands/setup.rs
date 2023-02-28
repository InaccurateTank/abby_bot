use std::{
	time::Duration,
	// collections::HashMap
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

// Less typing...
// macro_rules! create_options {
// 	($( $f:expr ),+) => {
// 		{
// 			let mut opts: Vec<serenity::CreateSelectMenuOption> = Vec::new();
// 			$(
// 				let name = stringify!($f).split('.').last().unwrap();
// 				opts.push(serenity::CreateSelectMenuOption::new(name[0..1].to_uppercase() + &name[1..], abby_utils::concat("enable_", name))
// 					.default_selection($f)
// 					.to_owned());
// 			)+
// 			opts
// 		}
// 	};
// }

// macro_rules! testmac {
// 	(&list:expr, $( $f:expr ),+) => {
// 		{
// 			$(
// 				let name = stringify!($f).split('.').last().unwrap();
// 				if list.contains(""name)
// 			)+
// 		}
// 	};
// }

// Server Features Struct
// #[derive(FromRow, Debug)]
// struct Server {
// 	srvid: i64,
// 	serious: bool,
// 	messages: bool,
// 	roles: bool
// }
// impl Server {
// 	// fn as_hashmap(&self) -> HashMap<&str, bool> {
// 	// 	let mut hash = HashMap::new();
// 	// 	hash.insert("serious", self.serious);
// 	// 	hash.insert("messages", self.messages);
// 	// 	hash.insert("roles", self.roles);
// 	// 	hash
// 	// }

// 	fn as_array(&self) -> [(&str, bool); 3] {
// 		[
// 			("serious", self.serious),
// 			("messages", self.messages),
// 			("roles", self.roles)
// 		]
// 	}

// 	fn selectmenu_options(&self) -> Vec<serenity::CreateSelectMenuOption> {
// 		let mut opts = Vec::new();
// 		// for (k, v) in self.as_hashmap() {
// 		// 	opts.push(serenity::CreateSelectMenuOption::new(k[0..1].to_uppercase() + &k[1..], concat("enable_", k))
// 		// 	.default_selection(v)
// 		// 	.to_owned());
// 		// }

// 		for (name, value) in self.as_array() {
// 			opts.push(serenity::CreateSelectMenuOption::new(name[0..1].to_uppercase() + &name[1..], concat("enable_", name))
// 				.default_selection(value)
// 				.to_owned());
// 		}
// 		opts
// 		// create_options!(self.serious, self.messages, self.roles)
// 	}

// 	// async fn toggle_features(&self, selections: Vec<String>) {
// 	// 	let original = self.as_hashmap();
// 	// 	let mut new = HashMap::new();
// 	// 	for key in original.keys() {
// 	// 		if selections.contains(&concat("enable_", key)) {
// 	// 			new.insert(*key, true);
// 	// 		} else {
// 	// 			new.insert(*key, false);
// 	// 		}
// 	// 	}
// 	// }
// }

/// Sets up various settings for the bot on the server.
#[poise::command(
	guild_only,
	slash_command,
	required_permissions="ADMINISTRATOR",
	ephemeral
)]
async fn bot(ctx: Context<'_>) -> Result<(), Error> {
	let id = *ctx.guild_id().unwrap().as_u64();

	let server_entry = query_as::<_, db_structs::Server>("SELECT * FROM servers WHERE srvid = ?;")
		.bind(id as i64)
		.fetch_one(&ctx.data().db)
		.await?;

	let reply = ctx.send(|b| {
		b.content("")
			.embed(|e| {
				e.title("Bot Configuration");
				e.color(serenity::utils::Color::new(663366));
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
							f.set_options(server_entry.as_selectmenuoptions())
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
				e.description("Processing changes, please wait...")
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

	// Hashmap fuckery
	// let mut new = HashMap::new();
	// {
		// let original = server_entry.as_hashmap();
		// for key in original.keys() {
		// 	if selected.contains(&concat("enable_", key)) {
		// 		new.insert(*key, true);
		// 	} else {
		// 		new.insert(*key, false);
		// 	}
		// }
	// let mut new = Vec::new();
	// {
	// 	let original = server_entry.as_array();
	// 	for (name, _) in original {
	// 		if selected.contains(&concat("enable_", name)) {
	// 			new.push((name, true))
	// 		} else {
	// 			new.push((name, false))
	// 		}
	// 	}
	// }
	let set_string = server_entry.as_array()
		.map(|(name, _)| {
			format!("{name} = {}", selected.contains(&concat("enable_", name)))
		}).join(",");
	// let mut set_string = String::new();
	// for (k, v) in new {
	// 	set_string.push_str(&format!("{k} = {v} "));
	// };

	// println!("UPDATE servers SET {set_string} WHERE srvid = {};", server_entry.srvid);

	sqlx::query(&format!("UPDATE servers SET {set_string} WHERE srvid = {};", server_entry.srvid))
		.execute(&ctx.data().db)
		.await?;
	// let a = format!("UPDATE servers SET {set_string} WHERE srvid = {};", server_entry.srvid);

	// println!("{a}");
	let updated = query_as::<_, db_structs::Server>("SELECT * FROM servers WHERE srvid = ?;")
		.bind(id as i64)
		.fetch_one(&ctx.data().db)
		.await?;
	reply.edit(ctx, |b| {
		b.content("")
			.embed(|e| {
				e.title("Feature Changes Confirmed!");
				e.description("Your new settings are:");
				for (name, value) in updated.as_array() {
					e.field(name[0..1].to_uppercase() + &name[1..], if value {"Enabled"} else {"Disabled"}, false);
				}
				e
			})
			.components(|f| f)
	})
	.await?;
	Ok(())
}

/// Creates a new role list.
#[poise::command(
	guild_only,
	slash_command,
	required_permissions="MANAGE_ROLES",
	ephemeral
)]
async fn roles(ctx: Context<'_>) -> Result<(), Error> {
	let roles = abby_utils::all_roles_select(ctx, None).await?;

	let reply = ctx.send(|b| {
		b.content("Please choose roles from the list.")
			.components(|c| {
				c.create_action_row(|row| {
					row.create_select_menu(|menu| {
						menu.custom_id("setup.rolelist");
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
		"setup.rolelist" => {
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

