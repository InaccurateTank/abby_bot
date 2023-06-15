use std::{
	collections::VecDeque,
	str::FromStr
};
use poise::serenity_prelude as serenity;
use abby_utils::{Error, db_structs};
use sqlx::{query, query_as};
use crate::{EMBED_STD, templates};

pub async fn mci_handler(ctx: &serenity::Context, data: &abby_utils::Data, mci: &serenity::MessageComponentInteraction) -> Result<(), Error> {
	let srv_features = query_as::<_, db_structs::Server>("SELECT * FROM servers WHERE srvid = ?;")
		.bind(*mci.guild_id.unwrap().as_u64() as i64)
		.fetch_one(&data.db)
		.await?;
	let mut mci_id: VecDeque<&str> = mci.data.custom_id.split_terminator('.').collect();
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

async fn roles_click(ctx: &serenity::Context, data: &abby_utils::Data, mci: &serenity::MessageComponentInteraction, id_vec: &mut VecDeque<&str>) -> Result<(), Error> {
	let srv_id = *mci.guild_id.unwrap().as_u64();
	let group = id_vec.pop_back().unwrap();
	let rolelist = query_as::<_, db_structs::RoleEntry>(&format!("SELECT * FROM roles_{srv_id} WHERE grp = ?;"))
		.bind(group)
		.fetch_all(&data.db)
		.await?;
	match id_vec.pop_front().unwrap() {
		// Rolelist Pick
		"pick" => {
			let mut rolelist_vec: Vec<serenity::Role> = rolelist.iter()
				.map(|f| serenity::RoleId(f.id as u64).to_role_cached(ctx).unwrap())
				.collect();
			rolelist_vec.sort_by(|a, b| {
				let a_l = a.name.to_lowercase();
				let b_l = b.name.to_lowercase();
				a_l.cmp(&b_l)
			});
			let member = mci.member.as_ref().unwrap();

			if member.permissions(ctx).unwrap().manage_roles() {
				mci.create_interaction_response(ctx, |i| {
					i.kind(serenity::InteractionResponseType::ChannelMessageWithSource);
					i.interaction_response_data(|m| {
						m.content("");
						m.ephemeral(true);
						m.set_embed(templates::state_embed(false, "For security reasons the role management feature only works on users without role management permissions. As you have these permissions, simply assign them yourself. If you cannot assign them to yourself then I can't assign them to anyone anyway."))
					})
				}).await?;
				return Ok(())
			}

			mci.create_interaction_response(&ctx, |i| {
				i.kind(serenity::InteractionResponseType::ChannelMessageWithSource);
				i.interaction_response_data(|m| {
					m.content("");
					m.ephemeral(true);
					m.embed(|e| {
						e.title("");
						e.color(EMBED_STD);
						e.description("Pick roles from the list below. Roles you already have will be preselected.")
					});
					m.components(|c| {
						c.create_action_row(|r| {
							r.create_select_menu(|menu| {
								menu.custom_id("addroles");
								menu.placeholder("Select a set of roles.");
								menu.min_values(0);
								menu.max_values(rolelist_vec.len() as u64);
								menu.options(|f| {
									let options: Vec<serenity::CreateSelectMenuOption> = rolelist_vec.iter().map(|r| {
										serenity::CreateSelectMenuOption::new(&r.name, r.id.as_u64())
											.default_selection(member.roles.contains(&r.id))
											.to_owned()
									}).collect();
									f.set_options(options)
								})
							})
						})
					})
				})
			}).await?;

			let secondary_mci = match mci.get_interaction_response(ctx).await?
				.await_component_interaction(ctx)
					.author_id(member.user.id)
					.timeout(std::time::Duration::from_secs(300))
					.await {
						Some(i) => {
							i
						},
						None => {
							mci.create_interaction_response(ctx, |f| {
								f.kind(serenity::InteractionResponseType::ChannelMessageWithSource);
								f.interaction_response_data(|m| {
									m.content("");
									m.ephemeral(true);
									m.set_embed(templates::state_embed(false, "Interaction timed out, please try again."))
								})
							}).await?;
							return Ok(())
						}
					};

			mci.edit_original_interaction_response(ctx, |i| {
				i.content("");
				i.set_embed(templates::processing_embed());
				i.components(|f| f)
			}).await?;

			let mut error_list = Vec::<String>::new();

			let selected_roles: Vec<serenity::RoleId> = secondary_mci.data.values.iter().map(|f| {
				serenity::RoleId::from_str(f).unwrap()
			}).collect();
			for r in rolelist_vec {
				let mem_contain = member.roles.contains(&r.id);
				let selected_contain = selected_roles.contains(&r.id);
				let current_users = rolelist.iter().find(|f| f.id == *r.id.as_u64() as i64).unwrap().users;
				if selected_contain && !mem_contain {
					let current_users = current_users.saturating_add(1);
					match member.to_owned()
						.add_role(ctx, r.id)
						.await {
						Ok(_) => {
							query(&format!("UPDATE roles_{srv_id} SET users = ? WHERE id = ?;"))
								.bind(current_users)
								.bind(*r.id.as_u64() as i64)
								.execute(&data.db)
								.await?;
						},
						Err(e) => error_list.push(format!("Error applying role \"{}\": {}", r.name, e))
					}
					//	.expect("Could not alter role");
					// query(&format!("UPDATE roles_{srv_id} SET users = ? WHERE id = ?;"))
					// 	.bind(current_users)
					// 	.bind(*r.id.as_u64() as i64)
					// 	.execute(&data.db)
					// 	.await?;
				} else if !selected_contain && mem_contain {
					let current_users = current_users.saturating_sub(1);
					match member.to_owned()
						.remove_role(ctx, r.id)
						.await {
						Ok(_) => {
							query(&format!("UPDATE roles_{srv_id} SET users = ? WHERE id = ?;"))
								.bind(current_users)
								.bind(*r.id.as_u64() as i64)
								.execute(&data.db)
								.await?;
						},
						Err(e) => error_list.push(format!("Error removing role \"{}\": {}", r.name, e))
					}
					// 	.expect("Could not alter role");
					// query(&format!("UPDATE roles_{srv_id} SET users = ? WHERE id = ?;"))
					// 	.bind(current_users)
					// 	.bind(*r.id.as_u64() as i64)
					// 	.execute(&data.db)
					// 	.await?;
				}
			}

			secondary_mci.create_interaction_response(ctx, |i| {
				i.kind(serenity::InteractionResponseType::UpdateMessage);
				i.interaction_response_data(|m| {
					m.content("");
					if error_list.is_empty() {
						m.set_embed(templates::state_embed(true, "Roles selected have been successfully applied to your server profile."));
					} else {
						m.set_embed(templates::state_embed(false, &format!("An error has been encountered on at least one role. Please contact the server administrator.\n\n{}", error_list.join("\n"))));
					}
					m.components(|c| c)
				})
			}).await?;

			let rolelist_new = query_as::<_, db_structs::RoleEntry>(&format!("SELECT * FROM roles_{srv_id} WHERE grp = ?;"))
				.bind(group)
				.fetch_all(&data.db)
				.await?;
			mci.message.to_owned().edit(ctx, |m| {
				m.set_embed(templates::rolelist_embed(ctx, group, rolelist_new))
			}).await?;
		},

		// Rolelist Edit
		"edit" => {
			if !mci.member.as_ref().unwrap().permissions(ctx).unwrap().manage_roles() {
				mci.create_interaction_response(ctx, |i| {
					i.kind(serenity::InteractionResponseType::ChannelMessageWithSource);
					i.interaction_response_data(|m| {
						m.content("");
						m.ephemeral(true);
						m.set_embed(templates::state_embed(false, "You do not have the required permissions for this."));
						m.components(|c| c)
					})
				}).await?;
				return Ok(());
			}

			// Select menu sorting
			let mut select: Vec<serenity::Role> = mci.guild_id.unwrap().to_guild_cached(ctx).unwrap().roles
				.into_values()
				.filter(|r| {
					abby_utils::role_filter(r)
				})
				.collect();
			select.sort_by(|a, b| {
				let a_l = a.name.to_lowercase();
				let b_l = b.name.to_lowercase();
				a_l.cmp(&b_l)
			});
			let select: Vec<serenity::CreateSelectMenuOption> = select.into_iter().filter_map(|r| {
					if r.name != "@everyone" {
						return Some(serenity::CreateSelectMenuOption::new(&r.name, r.id)
						.default_selection(rolelist.iter()
							.any(|f| r.id == serenity::RoleId(f.id as u64)))
						.to_owned())
					}
					None
				}).collect();

			mci.create_interaction_response(ctx, |i| {
				i.kind(serenity::InteractionResponseType::ChannelMessageWithSource);
				i.interaction_response_data(|m| {
					m.content("");
					m.ephemeral(true);
					m.embed(|e| {
						e.color(EMBED_STD);
						e.description("Please select a set of roles for the group below. Note that roles with permissions to modify the server are not available.")
					});
					m.components(|c| {
						c.create_action_row(|r| {
							r.create_select_menu(|menu| {
								menu.custom_id("rolelist.edit");
								menu.placeholder("Roles");
								menu.min_values(1);
								menu.max_values(select.len() as u64);
								menu.options(|f| {
									f.set_options(select)
								})
							})
						})
					})
				})
			}).await?;

			let secondary_mci = match mci.get_interaction_response(ctx).await?
				.await_component_interaction(ctx)
					.author_id(mci.member.as_ref().unwrap().user.id)
					.timeout(std::time::Duration::from_secs(300))
					.await {
						Some(i) => {
							i
						},
						None => {
							mci.create_interaction_response(ctx, |f| {
								f.kind(serenity::InteractionResponseType::ChannelMessageWithSource);
								f.interaction_response_data(|m| {
									m.content("");
									m.ephemeral(true);
									m.set_embed(templates::state_embed(false, "Interaction timed out, please try again."))
								})
							}).await?;
							return Ok(())
						}
					};

			mci.edit_original_interaction_response(ctx, |i| {
				i.content("");
				i.set_embed(templates::processing_embed());
				i.components(|f| f)
			}).await?;
			query(&format!("DELETE FROM roles_{srv_id} WHERE grp = ?;"))
				.bind(group)
				.execute(&data.db)
				.await.unwrap();
			for selected in &secondary_mci.data.values {
				query(&format!("INSERT INTO roles_{srv_id} (id, grp, users) VALUES(?, ?, ?);"))
					.bind(selected.parse::<i64>().unwrap())
					.bind(group)
					.bind(rolelist.iter().find_map(|f| if &f.id.to_string() == selected {Some(f.users)} else {None}).unwrap_or(0))
					.execute(&data.db)
					.await.unwrap();
			}

			let rolelist_new = query_as::<_, db_structs::RoleEntry>(&format!("SELECT * FROM roles_{srv_id} WHERE grp = ?;"))
				.bind(group)
				.fetch_all(&data.db)
				.await.unwrap();
			secondary_mci.create_interaction_response(ctx, |i| {
				i.kind(serenity::InteractionResponseType::UpdateMessage);
				i.interaction_response_data(|m| {
					m.content("");
					m.set_embed(templates::state_embed(true, &format!("Editing of group {group} has been recreated. The original interaction should be already updated.")));
					m.components(|c| c)
				})
			}).await?;
			mci.message.to_owned().edit(ctx, |m| {
				m.set_embed(templates::rolelist_embed(ctx, group, rolelist_new));
				m.set_components(templates::rolelist_components(group))
			}).await?;
		},
		_ => {
			println!("Unknown Interaction ID {}", &mci.data.custom_id);
			mci.create_interaction_response(ctx, |f| {
				f.kind(serenity::InteractionResponseType::ChannelMessageWithSource);
				f.interaction_response_data(|m| {
					m.content("");
					m.ephemeral(true);
					m.set_embed(templates::state_embed(false, "Unknown Interaction ID"))
				})
			}).await?;
		}
	}
	Ok(())
}
