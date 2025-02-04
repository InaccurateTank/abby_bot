use std::{
	collections::VecDeque,
	str::FromStr
};
use poise::serenity_prelude as serenity;
use sqlx::{query, query_as};
use crate::{
	structs::{
		db,
		misc
	},
	utils, templates,
	Error, Data,
	EMBED_STD
};

pub async fn mci_handler(
	ctx: &serenity::Context,
	data: &Data,
	mci: &serenity::ComponentInteraction
) -> Result<(), Error> {
	let srv_features = query_as::<_, db::ServerSettings>("SELECT * FROM server_settings WHERE id = ?;")
		.bind(mci.guild_id.unwrap_or_default().get() as i64)
		.fetch_one(&data.db)
		.await?;
	let mut mci_id: VecDeque<&str> = mci.data.custom_id.split_terminator('.').collect();

	// Interaction futureproof
	match mci_id.pop_front().ok_or_else(|| "Malformed interaction id.")? {
		"roles" => {
			if srv_features.roles {
				roles_click(ctx, data, mci, &mut mci_id).await?;
			}
		},
		_ => {}
	}
	Ok(())
}

async fn roles_click(
	ctx: &serenity::Context,
	data: &Data,
	mci: &serenity::ComponentInteraction,
	id_vec: &mut VecDeque<&str>
) -> Result<(), Error> {
	let Some(group_name) = id_vec.pop_back() else {
		// TODO: Better Errors
		return Err(Error::from("Malformed interaction id."))
	};

	let server = if let Some(r) = mci.guild_id
		.ok_or_else(|| "Not a Guild")?
		.to_guild_cached(ctx) {
		r.to_owned()
	} else {
		return Err(serenity::ModelError::GuildNotFound.into())
	};

	let db_group_roles = query_as::<_, db::Role>(&format!("SELECT * FROM roles WHERE server_id = ? AND group_name = ?;"))
		.bind(server.id.get() as i64)
		.bind(group_name)
		.fetch_all(&data.db)
		.await?;

	let Some(member) = mci.member.as_ref() else {
		// TODO: Better Errors
		return Err(serenity::ModelError::MemberNotFound.into())
	};
	let Some(channel) = mci.channel_id
		.to_channel(ctx)
		.await?
		.guild() else {
		// TODO: Better Errors
		return Err(Error::from("Not a GuildChannel"))
	};

	match id_vec.pop_front().ok_or_else(|| "Malformed interaction id.")? {
		// Rolelist Pick
		"pick" => {
			// TODO: Standardize Error
			if server.user_permissions_in(&channel, &member).manage_roles() {
				mci.create_response(ctx, serenity::CreateInteractionResponse::Message(serenity::CreateInteractionResponseMessage::new()
					.ephemeral(true)
					.embed(templates::state_embed(false, "For security reasons the role management feature only works on users without role management permissions. As you have these permissions, simply assign them yourself. If you cannot assign them to yourself then I can't assign them to anyone anyway."))
				)).await?;
				return Ok(())
			}

			let mut roles_list = db_group_roles.into_iter()
				.map(|r| {
					misc::RoleVitals::new(r.role_id, &server)
				}).collect::<Result<Vec<misc::RoleVitals>, Error>>()?;
			roles_list.sort_by(|a, b| {
				let a_l = a.name.to_lowercase();
				let b_l = b.name.to_lowercase();
				a_l.cmp(&b_l)
			});

			// Vec of RoleIds with their names
			// let mut guild_roles: Vec<(&serenity::RoleId, String)> = db_group_roles.iter()
			// 	.map(|r| {
			// 		let rname = server.roles
			// 			.get(&r.role_id)
			// 			.unwrap()
			// 			.name
			// 			.to_owned();
			// 		(&r.role_id, rname)
			// 	}
			// ).collect();
			// guild_roles.sort_by(|(_, a), (_, b)| {
			// 	let a_l = a.to_lowercase();
			// 	let b_l = b.to_lowercase();
			// 	a_l.cmp(&b_l)
			// });

			// Picking response
			mci.create_response(ctx, serenity::CreateInteractionResponse::Message(serenity::CreateInteractionResponseMessage::new()
				.ephemeral(true)
				.embed(serenity::CreateEmbed::new()
					.color(EMBED_STD)
					.description("Pick roles from the list below. Roles you already have will be preselected.")
				).components(vec![
					serenity::CreateActionRow::SelectMenu(serenity::CreateSelectMenu::new("choose_roles", serenity::CreateSelectMenuKind::String { options: roles_list.iter()
						.map(|r| {
							serenity::CreateSelectMenuOption::new(&r.name, r.id.get().to_string())
								.default_selection(member.roles.contains(&r.id))
							}).collect()
						}).placeholder("Select a set of roles.")
						.min_values(0)
						.max_values(roles_list.len() as u8))
				])
			)).await?;

			// Wait for response
			let response_message = mci.get_response(ctx).await?;
			let sec_mci = match response_message.await_component_interaction(ctx)
				.author_id(member.user.id)
				.timeout(std::time::Duration::from_secs(300))
				.await {
				Some(i) => i,
				None => {
					mci.edit_response(ctx, serenity::EditInteractionResponse::new()
						.embed(templates::state_embed(false, "Interaction timed out, please try again."))
						.components(Vec::new())
					).await?;
					return Ok(())
				}
			};

			// Processing message
			mci.edit_response(ctx, serenity::EditInteractionResponse::new()
				.embed(templates::processing_embed())
				.components(Vec::new())
			).await?;

			// Vector of selected roles
			let selected_roles = match &sec_mci.data.kind {
				serenity::ComponentInteractionDataKind::StringSelect { values } => values.into_iter()
					.map(|s| serenity::RoleId::from_str(s).map_err(|e| e.into()))
					.collect::<Result<Vec<serenity::RoleId>, Error>>()?,
				_ => {
					sec_mci.create_response(ctx, serenity::CreateInteractionResponse::UpdateMessage(serenity::CreateInteractionResponseMessage::new()
						.embed(templates::state_embed(false, "Somehow recieved wrong interaction, please report this."))
						.components(Vec::new())
					)).await?;
					return Ok(())
				}
			};

			// let add_to: Vec<&RoleVitals> = selected_roles.iter()
			// 	.filter(|f| !member.roles.contains(&f.id))
			// 	.collect();

			// let remove_from: Vec<&RoleVitals> = member.roles
			// 	.iter()
			// 	.filter_map(|f| {
			// 		let Some(invert) = selected_roles.iter().find(|i| i.id == *f) else {
			// 			return None
			// 		};
			// 		Some(invert)
			// 	})
			// 	.collect();

			let mut error_list = Vec::<String>::new();
			for r in &mut roles_list {
				// Finish processing user counts
				r.users.solve().await?;
				let member_contain = member.roles.contains(&r.id);
				let selected_contain = selected_roles.iter()
					.any(|f| f == &r.id);

				// If role is selected but not in member roles
				if selected_contain && !member_contain {
					// Add Role to Member
					match member.to_owned().add_role(ctx, r.id).await {
						Ok(_) => {r.users.increment()?;},
						Err(e) => error_list.push(format!("Error applying role \"{}\": {}", r.name, e))
					}
				// If role isn't selected and is in member roles
				} else if !selected_contain && member_contain {
					// Remove Role from Member
					match member.to_owned().remove_role(ctx, r.id).await {
						Ok(_) => {r.users.decrement()?;},
						Err(e) => error_list.push(format!("Error removing role \"{}\": {}", r.name, e))
					}
				}
			}

			sec_mci.create_response(ctx, serenity::CreateInteractionResponse::UpdateMessage(serenity::CreateInteractionResponseMessage::new()
				.ephemeral(true)
				.embed(
					if error_list.is_empty() {
						templates::state_embed(true, "Roles selected have been successfully applied to your server profile.")
					} else {
						templates::state_embed(false, &format!("An error has been encountered on at least one role. Please contact the server administrator.\n\n{}", error_list.join("\n")))
					}
				)
			)).await?;

			roles_list.sort_by(|a, b| {
				let a_l = a.name.to_lowercase();
				let b_l = b.name.to_lowercase();
				a_l.cmp(&b_l)
			});

			// let role_list_new: Vec<db::Role> = query_as::<_, db::Role>(&format!("SELECT * FROM roles WHERE server_id = ? AND group_name = ?;"))
			// 	.bind(server.id.get() as i64)
			// 	.bind(group_name)
			// 	.fetch_all(&data.db)
			// 	.await?;
			mci.message.to_owned().edit(ctx, serenity::EditMessage::new()
				.embed(templates::rolelist_embed(group_name, &roles_list)?)
			).await?;
		},

		// Rolelist Edit
		"edit" => {
			if !server.user_permissions_in(&channel, &member).manage_roles() {
				mci.create_response(ctx, serenity::CreateInteractionResponse::Message(serenity::CreateInteractionResponseMessage::new()
					.ephemeral(true)
					.embed(templates::state_embed(false, "You do not have the required permissions for this."))
				)).await?;
				return Ok(());
			}

			// Select menu sorting
			let mut server_roles: Vec<&serenity::Role> = server.roles
				.values()
				.filter(|r| utils::role_filter(r))
				.collect();
			server_roles.sort_by(|a, b| {
				let a_l = a.name.to_lowercase();
				let b_l = b.name.to_lowercase();
				a_l.cmp(&b_l)
			});

			mci.create_response(ctx, serenity::CreateInteractionResponse::Message(serenity::CreateInteractionResponseMessage::new()
				.ephemeral(true)
				.embed(serenity::CreateEmbed::new()
					.color(EMBED_STD)
					.description("Please select a set of roles for the group below. Note that roles with permissions to modify the server are not available.")
				).components(vec![
					serenity::CreateActionRow::SelectMenu(serenity::CreateSelectMenu::new("rolelist.edit", serenity::CreateSelectMenuKind::String { options: server_roles.iter()
						.map(|r| {
							serenity::CreateSelectMenuOption::new(&r.name, r.id.to_string())
								.default_selection(db_group_roles.iter().any(|f| r.id == f.role_id))
							}).collect()
						}).placeholder("Roles")
						.min_values(1)
						.max_values(server_roles.len() as u8))
				])
			)).await?;

			let response_message = mci.get_response(ctx).await?;
			let sec_mci = match response_message.await_component_interaction(ctx)
				.author_id(member.user.id)
				.timeout(std::time::Duration::from_secs(300))
				.await {
				Some(i) => i,
				None => {
					mci.edit_response(ctx, serenity::EditInteractionResponse::new()
						.embed(templates::state_embed(false, "Interaction timed out, please try again."))
						.components(Vec::new())
					).await?;
					return Ok(())
				}
			};

			mci.edit_response(ctx, serenity::EditInteractionResponse::new()
				.embed(templates::processing_embed())
				.components(Vec::new())
			).await?;

			let selected_roles = if let serenity::ComponentInteractionDataKind::StringSelect { values } = &sec_mci.data.kind {
				values.into_iter()
				.map(|s| misc::RoleVitals::new(serenity::RoleId::from_str(s)?, &server))
				.collect::<Result<Vec<misc::RoleVitals>, Error>>()?
			} else {
				sec_mci.create_response(ctx, serenity::CreateInteractionResponse::UpdateMessage(serenity::CreateInteractionResponseMessage::new()
					.embed(templates::state_embed(false, "Somehow recieved wrong interaction, please report this."))
					.components(Vec::new())
				)).await?;
				return Ok(())
			};

			query("DELETE FROM roles WHERE group_name = ?;")
				.bind(group_name)
				.execute(&data.db)
				.await?;
			for entry in &selected_roles {
				query("INSERT INTO roles (server_id, group_name, role_id) VALUES(?, ?, ?);")
					.bind(server.id.get() as i64)
					.bind(group_name)
					.bind(entry.id.get() as i64)
					.execute(&data.db)
					.await?;
			}

			sec_mci.create_response(ctx, serenity::CreateInteractionResponse::UpdateMessage(serenity::CreateInteractionResponseMessage::new()
				.ephemeral(true)
				.embed(templates::state_embed(true, &format!("Editing of group {group_name} has been completed. The original message should update shortly.")))
			)).await?;

			mci.message.to_owned().edit(ctx, serenity::EditMessage::new()
				.embed(templates::rolelist_embed(group_name, &selected_roles)?)
			).await?;
		},

		// Unknown
		_ => {
			println!("Unknown Interaction ID {}", &mci.data.custom_id);
			mci.create_response(ctx, serenity::CreateInteractionResponse::Message(serenity::CreateInteractionResponseMessage::new()
				.ephemeral(true)
				.embed(templates::state_embed(false, "Unknown Interaction ID"))
			)).await?;
		}
	}
	Ok(())
}
