use std::{
	collections::VecDeque,
	str::FromStr
};
use poise::serenity_prelude as serenity;
use abby_utils::{Error, db_structs, inter_err};
use sqlx::{query, query_as};
use crate::{EMBED_STD, EMBED_WAIT};

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
		"pick" => {
			let rolelist_vec: Vec<serenity::Role> = rolelist.iter()
				.map(|f| serenity::RoleId(f.id as u64).to_role_cached(ctx).unwrap())
				.collect();
			let member = mci.member.as_ref().unwrap();

			mci.create_interaction_response(&ctx, |i| {
				i.kind(serenity::InteractionResponseType::ChannelMessageWithSource);
				i.interaction_response_data(|m| {
					m.content("");
					m.ephemeral(true);
					m.embed(|e| {
						e.title("Test");
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
									m.content("Interaction timed out, please try again.");
									m.ephemeral(true)
								})
							}).await?;
							return Ok(())
						}
					};

			mci.edit_original_interaction_response(ctx, |i| {
				i.content("");
				i.embed(|e| {
					e.color(EMBED_WAIT);
					e.description("Processing, please wait...")
				});
				i.components(|f| f)
			}).await?;

			let selected_roles: Vec<serenity::RoleId> = secondary_mci.data.values.iter().map(|f| {
				serenity::RoleId::from_str(f).unwrap()
			}).collect();
			for r in rolelist_vec {
				let mem_contain = member.roles.contains(&r.id);
				if selected_roles.contains(&r.id) && !mem_contain {
					member.to_owned()
						.add_role(ctx, r.id)
						.await
						.expect("Could not alter role");
					query(&format!("UPDATE roles_{srv_id} SET users = ? WHERE id = ?;"))
						.bind(
							rolelist.iter().find(|f| {
								f.id == *r.id.as_u64() as i64
							}).unwrap()
								.users
								.saturating_add(1)
						)
						.bind(*r.id.as_u64() as i64)
						.execute(&data.db)
						.await?;
				} else if mem_contain {
					member.to_owned()
						.remove_role(ctx, r.id)
						.await
						.expect("Could not alter role");
					query(&format!("UPDATE roles_{srv_id} SET users = ? WHERE id = ?;"))
						.bind(
							rolelist.iter().find(|f| {
								f.id == *r.id.as_u64() as i64
							}).unwrap()
								.users
								.saturating_sub(1)
						)
						.bind(*r.id.as_u64() as i64)
						.execute(&data.db)
						.await?;
				}
			}

			secondary_mci.create_interaction_response(ctx, |i| {
				i.kind(serenity::InteractionResponseType::UpdateMessage);
				i.interaction_response_data(|m| {
					m.content("");
					m.embed(|e| {
						e.title("Roles Applied!");
						e.color(EMBED_STD);
						e.description("Roles from the selected have been successfully applied to your server profile.")
					});
					m.components(|c| c)
				})
			}).await?;

			let rolelist_new = query_as::<_, db_structs::RoleEntry>(&format!("SELECT * FROM roles_{srv_id} WHERE grp = ?;"))
				.bind(group)
				.fetch_all(&data.db)
				.await?;
			mci.message.to_owned().edit(ctx, |m| {
				m.embed(|e| {
					e.title(group);
					e.color(serenity::utils::Color::from_rgb(102, 51, 102));
					let mut name_str = String::new();
					let mut user_str = String::new();
					for r in rolelist_new {
						name_str = abby_utils::concat(&name_str, &format!("{}\n", serenity::RoleId(r.id as u64).to_role_cached(ctx).unwrap().name));
						user_str = abby_utils::concat(&user_str, &format!("{}\n", r.users));
					}
					name_str = name_str.trim_end_matches('\n').to_string();
					user_str = user_str.trim_end_matches('\n').to_string();
					e.field("Name", name_str, true);
					e.field("Users", user_str, true)
				})
			}).await?;
		},
		"edit" => {
			mci.create_interaction_response(ctx, |i| {
				i.kind(serenity::InteractionResponseType::ChannelMessageWithSource);
				i.interaction_response_data(|m| {
					m.content("Todo")
				})
			}).await?;
		},
		"remove" => {
			query(&format!("DELETE FROM roles_{srv_id} WHERE grp = ?;"))
				.bind(group)
				.execute(&data.db)
				.await?;
			mci.message.delete(ctx).await?;
			mci.create_interaction_response(ctx, |i| {
				i.kind(serenity::InteractionResponseType::ChannelMessageWithSource);
				i.interaction_response_data(|m| {
					m.content("");
					m.ephemeral(true);
					m.embed(|e| {
						e.title(":white_check_mark: Success :white_check_mark:");
						e.color(EMBED_STD);
						e.description(format!("Role group {group} has been deleted."))
					});
					m.components(|c| c)
				})
			}).await?;
		},
		_ => {
			inter_err(":warning: Unknown Interaction ID :warning:", &format!("Unknown Interaction ID {}", &mci.data.custom_id), ctx, mci).await?;
		}
	}
	Ok(())
}
		// "roleadd" => {
		// 	let mut remove = "Added";
		// 	let rid = serenity::RoleId::from_str(back)
		// 		.expect("Could not parse RoleId");

		// 	for role in &mci.member.as_ref().unwrap().roles {
		// 		if role == &rid {
		// 			remove = "Removed";
		// 		}
		// 	}

		// 	if remove == "Removed" {
		// 		mci.member
		// 			.clone()
		// 			.expect("Could not find Interaction Member")
		// 			.remove_role(&ctx, rid)
		// 			.await?;
		// 	} else {
		// 		mci.member
		// 			.clone()
		// 			.expect("Could not find Interaction Member")
		// 			.add_role(&ctx, rid)
		// 			.await?
		// 	}

		// 	// ctx.cache.guild(mci.guild_id.unwrap()).unwrap().roles;
		// 	mci.create_interaction_response(&ctx, |f| {
		// 		f.kind(serenity::InteractionResponseType::ChannelMessageWithSource).interaction_response_data(|d| {
		// 				d.ephemeral(true);
		// 				d.content(format!("Role {back} has been {remove}."))
		// 			})
		// 		}).await?;
		// }
		// "edit" => {
		// 	// let mut list: Vec<String> = Vec::new();
		// 	for row in &mci.message.components {
		// 		for arc in &row.components {
		// 			if let serenity::ActionRowComponent::Button(button) = arc {
		// 				let id = &**button
		// 					.custom_id
		// 					.as_ref()
		// 					.unwrap();
		// 				let (f, b) = id
		// 					.split_once('.')
		// 					.unwrap();
		// 				if f == "roleadd" {
		// 					println!("{b}");
		// 					// list.push(b.to_string());
		// 				}
		// 			}
		// 		}
		// 	}
		// },
