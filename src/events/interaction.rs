use std::{collections::VecDeque, str::FromStr};
use poise::serenity_prelude as serenity;
use sqlx::{query, query_as};
use crate::{
	structs::db,
	utils, templates,
	Error, Data,
	EMBED_STD
};

pub async fn mci_handler(ctx: &serenity::Context, data: &Data, mci: &serenity::ComponentInteraction) -> Result<(), Error> {
	let srv_features = query_as::<_, db::Server>("SELECT * FROM servers WHERE srvid = ?;")
		.bind(mci.guild_id.unwrap().get() as i64)
		.fetch_one(&data.db)
		.await
		.unwrap();
	let mut mci_id: VecDeque<&str> = mci.data.custom_id.split_terminator('.').collect();

	// Interaction futureproof
	match mci_id.pop_front().unwrap() {
		"roles" => {
			if srv_features.roles {
				roles_click(ctx, data, mci, &mut mci_id).await?;
			}
		},
		_ => {}
	}
	Ok(())
}

async fn roles_click(ctx: &serenity::Context, data: &Data, mci: &serenity::ComponentInteraction, id_vec: &mut VecDeque<&str>) -> Result<(), Error> {
	let srv_id = mci.guild_id.unwrap().get();
	let group = id_vec.pop_back().unwrap();
	let group_roles = query_as::<_, db::RoleEntry>(&format!("SELECT * FROM roles_{srv_id} WHERE grp = ?;"))
		.bind(group)
		.fetch_all(&data.db)
		.await?;
	let member = mci.member
		.as_ref()
		.unwrap();
	let channel = mci.channel_id
		.to_channel(ctx)
		.await?
		.guild()
		.unwrap();
	let guild = mci.guild_id
		.unwrap()
		.to_guild_cached(ctx)
		.unwrap()
		.clone();
	match id_vec.pop_front().unwrap() {
		// Rolelist Pick
		"pick" => {
			if guild.user_permissions_in(&channel, &member).manage_roles() {
				mci.create_response(ctx, serenity::CreateInteractionResponse::Message(serenity::CreateInteractionResponseMessage::new()
					.ephemeral(true)
					.embed(templates::state_embed(false, "For security reasons the role management feature only works on users without role management permissions. As you have these permissions, simply assign them yourself. If you cannot assign them to yourself then I can't assign them to anyone anyway."))
				)).await?;
				return Ok(())
			}

			let mut role_list: Vec<(serenity::RoleId, String)> = group_roles.iter()
				.map(|r| {
					let rid = r.extract_roleid();
					let rname = serenity::GuildId::new(srv_id).to_guild_cached(ctx).unwrap().roles.get(&rid).unwrap().name.clone();
					(rid, rname)
				}).collect();
			role_list.sort_by(|(_, a), (_, b)| {
					let a_l = a.to_lowercase();
					let b_l = b.to_lowercase();
					a_l.cmp(&b_l)
				});

			mci.create_response(ctx, serenity::CreateInteractionResponse::Message(serenity::CreateInteractionResponseMessage::new()
				.ephemeral(true)
				.embed(serenity::CreateEmbed::new()
					.color(EMBED_STD)
					.description("Pick roles from the list below. Roles you already have will be preselected.")
				).components(vec![
					serenity::CreateActionRow::SelectMenu(serenity::CreateSelectMenu::new("choose_roles", serenity::CreateSelectMenuKind::String { options: role_list.iter()
						.map(|(rid, n)| {
							serenity::CreateSelectMenuOption::new(n, rid.get().to_string())
								.default_selection(member.roles.contains(rid))
							}).collect()
						}).placeholder("Select a set of roles.")
						.min_values(0)
						.max_values(role_list.len() as u8))
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

			let selected_roles: Vec<serenity::RoleId> = match &sec_mci.data.kind {
				serenity::ComponentInteractionDataKind::StringSelect { values } => values.iter()
					.map(|s| serenity::RoleId::from_str(s).unwrap())
					.collect(),
				_ => {
					sec_mci.create_response(ctx, serenity::CreateInteractionResponse::UpdateMessage(serenity::CreateInteractionResponseMessage::new()
						.embed(templates::state_embed(false, "Somehow recieved wrong interaction, please report this."))
						.components(Vec::new())
					)).await?;
					return Ok(())
				}
			};

			let mut error_list = Vec::<String>::new();
			for (rid, name) in role_list {
				let mem_contain = member.roles.contains(&rid);
				let selected_contain = selected_roles.contains(&rid);
				let current_users = group_roles.iter().find(|f| f.id == rid.get() as i64).unwrap().users;
				if selected_contain && !mem_contain {
					// Add Role to Member
					let current_users = current_users.saturating_add(1);
					match member.to_owned()
						.add_role(ctx, rid)
						.await {
						Ok(_) => {
							query(&format!("UPDATE roles_{srv_id} SET users = ? WHERE id = ?;"))
								.bind(current_users)
								.bind(rid.get() as i64)
								.execute(&data.db)
								.await
								.unwrap();
						},
						Err(e) => error_list.push(format!("Error applying role \"{name}\": {e}"))
					}
				} else if !selected_contain && mem_contain {
					// Remove Role from Member
					let current_users = current_users.saturating_sub(1);
					match member.to_owned()
						.remove_role(ctx, rid)
						.await {
						Ok(_) => {
							query(&format!("UPDATE roles_{srv_id} SET users = ? WHERE id = ?;"))
								.bind(current_users)
								.bind(rid.get() as i64)
								.execute(&data.db)
								.await
								.unwrap();
						},
						Err(e) => error_list.push(format!("Error removing role \"{name}\": {e}"))
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

			let role_list_new: Vec<db::RoleEntry> = query_as::<_, db::RoleEntry>(&format!("SELECT * FROM roles_{srv_id} WHERE grp = ?;"))
				.bind(group)
				.fetch_all(&data.db)
				.await
				.unwrap();
			mci.message.to_owned().edit(ctx, serenity::EditMessage::new()
				.embed(templates::rolelist_embed(ctx, group, role_list_new, serenity::GuildId::new(srv_id)))
			).await?;
		},

		// Rolelist Edit
		"edit" => {
			if !member.permissions(ctx).unwrap().manage_roles() {
				mci.create_response(ctx, serenity::CreateInteractionResponse::Message(serenity::CreateInteractionResponseMessage::new()
					.ephemeral(true)
					.embed(templates::state_embed(false, "You do not have the required permissions for this."))
				)).await?;
				return Ok(());
			}

			// Select menu sorting
			let mut roles_list: Vec<serenity::Role> = mci.guild_id
				.unwrap()
				.roles(ctx)
				.await?
				.into_values()
				.filter(|r| {
					utils::role_filter(r)
				})
				.collect();
			roles_list.sort_by(|a, b| {
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
					serenity::CreateActionRow::SelectMenu(serenity::CreateSelectMenu::new("rolelist.edit", serenity::CreateSelectMenuKind::String { options: roles_list.iter()
						.map(|r| {
							serenity::CreateSelectMenuOption::new(&r.name, r.id.to_string())
								.default_selection(group_roles.iter().any(|f| r.id == serenity::RoleId::new(f.id as u64)))
							}).collect()
						}).placeholder("Roles")
						.min_values(1)
						.max_values(roles_list.len() as u8))
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

			let selected = match &sec_mci.data.kind {
				serenity::ComponentInteractionDataKind::StringSelect { values } => values,
				_ => {
					sec_mci.create_response(ctx, serenity::CreateInteractionResponse::UpdateMessage(serenity::CreateInteractionResponseMessage::new()
						.embed(templates::state_embed(false, "Somehow recieved wrong interaction, please report this."))
						.components(Vec::new())
					)).await?;
					return Ok(())
				}
			};

			query(&format!("DELETE FROM roles_{srv_id} WHERE grp = ?;"))
				.bind(group)
				.execute(&data.db)
				.await
				.unwrap();
			for entry in selected {
				query(&format!("INSERT INTO roles_{srv_id} (id, grp, users) VALUES(?, ?, ?);"))
					.bind(entry.parse::<i64>().unwrap())
					.bind(group)
					.bind(group_roles.iter().find_map(|f| if &f.id.to_string() == entry {Some(f.users)} else {None}).unwrap_or(0))
					.execute(&data.db)
					.await
					.unwrap();
			}

			let role_list_new = query_as::<_, db::RoleEntry>(&format!("SELECT * FROM roles_{srv_id} WHERE grp = ?;"))
				.bind(group)
				.fetch_all(&data.db)
				.await
				.unwrap();

			sec_mci.create_response(ctx, serenity::CreateInteractionResponse::UpdateMessage(serenity::CreateInteractionResponseMessage::new()
				.ephemeral(true)
				.embed(templates::state_embed(true, &format!("Editing of group {group} has been completed. The original message should update shortly.")))
			)).await?;

			mci.message.to_owned().edit(ctx, serenity::EditMessage::new()
				.embed(templates::rolelist_embed(ctx, group, role_list_new, serenity::GuildId::new(srv_id)))
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
