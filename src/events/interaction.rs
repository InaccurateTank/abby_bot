use std::{
	collections::VecDeque,
	str::FromStr
};
use color_eyre::Result;
use poise::serenity_prelude as serenity;
use tracing::instrument;
use crate::{
	colors, database, error::{BotError, UserError}, structs, templates, utils, Data
};

pub async fn mci_handler(
	ctx: &serenity::Context,
	data: &Data,
	mci: &serenity::ComponentInteraction
) -> Result<()> {
	let srv_features = database::GuildSettings::from_query(mci.guild_id.unwrap_or_default(), &data.db).await?;
	let mut mci_id: VecDeque<&str> = mci.data.custom_id.split_terminator('.').collect();

	// Interaction futureproof
	match mci_id.pop_front().ok_or(BotError::MalformedInteraction)? {
		"roles" => {
			if srv_features.roles {
				roles_click(ctx, data, mci, &mut mci_id).await?;
			}
		},
		_ => {}
	}
	Ok(())
}

#[instrument(skip_all)]
async fn roles_click(
	ctx: &serenity::Context,
	data: &Data,
	mci: &serenity::ComponentInteraction,
	id_vec: &mut VecDeque<&str>
) -> Result<()> {
	let Some(group_name) = id_vec.pop_back() else {
		return Err(BotError::MalformedInteraction.into())
	};

	let guild = mci.guild_id
		.ok_or(BotError::InteractionNotInGuild)?
		.to_guild_cached(ctx)
		.ok_or(serenity::Error::Model(serenity::ModelError::GuildNotFound))?
		.to_owned();

	let db_group_roles = database::roles_from_query(guild.id, group_name, &data.db).await?;

	let Some(member) = mci.member.as_ref() else {
		return Err(serenity::ModelError::MemberNotFound.into())
	};
	let Some(channel) = mci.channel_id
		.to_channel(ctx)
		.await?
		.guild() else {
		return Err(BotError::InteractionNotInGuild.into())
	};

	match id_vec.pop_front().ok_or(BotError::MalformedInteraction)? {
		// Rolelist Pick
		"pick" => {
			if guild.user_permissions_in(&channel, member).manage_roles() {
				mci.create_response(ctx, serenity::CreateInteractionResponse::Message(serenity::CreateInteractionResponseMessage::new()
					.ephemeral(true)
					.embed(templates::status::warning(
						None,
						"For security reasons the role management feature only works on users without role management permissions. As you have these permissions simply assign them yourself. If you can't assign them to yourself I can't assign them to anyone regardless of rank."
					))
				)).await?;
				return Ok(())
			}

			let mut roles_list = db_group_roles.into_iter()
				.map(|r| {
					structs::RoleVitals::new(r, &guild)
				}).collect::<Result<Vec<structs::RoleVitals>>>()?;
			roles_list.sort_by(|a, b| {
				let a_l = a.name.to_lowercase();
				let b_l = b.name.to_lowercase();
				a_l.cmp(&b_l)
			});

			// Picking response
			mci.create_response(ctx, serenity::CreateInteractionResponse::Message(serenity::CreateInteractionResponseMessage::new()
				.ephemeral(true)
				.embed(serenity::CreateEmbed::new()
					.color(colors::INFO)
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
			let Some(sec_mci) = response_message.await_component_interaction(ctx)
				.author_id(member.user.id)
				.timeout(std::time::Duration::from_secs(300))
				.await else
			{
				response_message.delete(ctx).await?;
				return Err(BotError::InteractionTimedOut.into())
			};

			// Processing message
			mci.edit_response(ctx, serenity::EditInteractionResponse::new()
				.embed(templates::status::processing())
				.components(Vec::new())
			).await?;

			// Vector of selected roles
			let selected_roles = match &sec_mci.data.kind {
				serenity::ComponentInteractionDataKind::StringSelect { values } => values.iter()
					.map(|s| serenity::RoleId::from_str(s).map_err(|e| e.into()))
					.collect::<Result<Vec<serenity::RoleId>>>()?,
				_ => {
					sec_mci.create_response(ctx, serenity::CreateInteractionResponse::Acknowledge).await?;
					response_message.delete(ctx).await?;
					return Err(BotError::WrongInteraction.into())
				}
			};

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
						templates::status::success(
							None,
							"Roles selected have been successfully applied to your server profile."
						)
					} else {
						templates::status::error(
							None,
							format!("An error has been encountered on at least one role. Please contact the server administrator.\n\n{}", error_list.join("\n"))
						)
					}
				)
			)).await?;

			mci.message.to_owned().edit(ctx, serenity::EditMessage::new()
				.embed(templates::rolelist::embed(group_name, &mut roles_list).await?)
			).await?;
		},

		// Rolelist Edit
		"edit" => {
			if !guild.user_permissions_in(&channel, member).manage_roles() {
				let permissions = serenity::Permissions::MANAGE_ROLES;
				mci.create_response(ctx, serenity::CreateInteractionResponse::Message(serenity::CreateInteractionResponseMessage::new()
					.ephemeral(true)
					.embed(templates::status::warning(
						None,
						format!("The following permissions are required to use that component:\n```{}```", permissions.get_permission_names().join("\n").to_ascii_uppercase())
					))
				)).await?;
				return Err(UserError(BotError::InteractionMissingPermissions(permissions).into()).into())
			}

			// Select menu sorting
			let mut server_roles: Vec<&serenity::Role> = guild.roles
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
					.color(colors::INFO)
					.description("Please select a set of roles for the group below. Note that roles with permissions to modify the server are not available.")
				).components(vec![
					serenity::CreateActionRow::SelectMenu(serenity::CreateSelectMenu::new("rolelist.edit", serenity::CreateSelectMenuKind::String { options: server_roles.iter()
						.map(|r| {
							serenity::CreateSelectMenuOption::new(&r.name, r.id.to_string())
								.default_selection(db_group_roles.iter().any(|f| &r.id == f))
							}).collect()
						}).placeholder("Roles")
						.min_values(1)
						.max_values(server_roles.len() as u8))
				])
			)).await?;

			let response_message = mci.get_response(ctx).await?;
			let Some(sec_mci) = response_message.await_component_interaction(ctx)
				.author_id(member.user.id)
				.timeout(std::time::Duration::from_secs(300))
				.await else
			{
				response_message.delete(ctx).await?;
				return Err(BotError::InteractionTimedOut.into())
			};

			// Processing message
			mci.edit_response(ctx, serenity::EditInteractionResponse::new()
				.embed(templates::status::processing())
				.components(Vec::new())
			).await?;

			let mut selected_roles = match &sec_mci.data.kind {
				serenity::ComponentInteractionDataKind::StringSelect { values } => values.iter()
					.map(|s| structs::RoleVitals::new(
						serenity::RoleId::from_str(s)?,
							&guild
						)
					)
					.collect::<Result<Vec<structs::RoleVitals>>>()?,
				_ => {
					sec_mci.create_response(ctx, serenity::CreateInteractionResponse::Acknowledge).await?;
					response_message.delete(ctx).await?;
					return Err(BotError::WrongInteraction.into())
				}
			};

			sqlx::query("DELETE FROM roles WHERE group_name = ?;")
				.bind(group_name)
				.execute(&data.db)
				.await?;
			for entry in &selected_roles {
				sqlx::query("INSERT INTO roles (guild_id, group_name, role_id) VALUES(?, ?, ?);")
					.bind(guild.id.get() as i64)
					.bind(group_name)
					.bind(entry.id.get() as i64)
					.execute(&data.db)
					.await?;
			}

			sec_mci.create_response(ctx, serenity::CreateInteractionResponse::UpdateMessage(serenity::CreateInteractionResponseMessage::new()
				.ephemeral(true)
				.embed(
					templates::status::success(
						None,
						format!("Editing of group {group_name} has been completed. The original message should update shortly.")
					)
				)
			)).await?;

			mci.message.to_owned().edit(ctx, serenity::EditMessage::new()
				.embed(templates::rolelist::embed(group_name, &mut selected_roles).await?)
			).await?;
		},

		// Unknown
		_ => return Err(BotError::WrongInteraction.into())
	}
	Ok(())
}
