use std::str::FromStr;
use poise::serenity_prelude as serenity;
use sqlx::{
	query,
	query_as,
	query_scalar
};
use crate::{
	structs::{db, misc},
	utils, templates,
	Context, Error,
	EMBED_STD
};

/// Creates or deletes a list of roles to select from.
///
/// ```Subcommands:
/// 	create      Creates a new role list.
/// 	delete      Deletes a role list.
/// ```
/// All subcommands are ephemeral and require the `Manage Roles` permission.
/// Using `/setup roles` before using the create subcommend is *highly* recommended. In some cases it may even be required due to errors in permissions on default channels.
#[poise::command(
	guild_only,
	slash_command,
	required_permissions="MANAGE_ROLES",
	category="Administration",
	ephemeral,
	subcommands("create", "delete")
)]
pub async fn rolelist(
	ctx: Context<'_>
) -> Result<(), Error> {
	ctx.say("You shouldn't be here?").await?;
	Ok(())
}

async fn autocomplete_groups<'a>(
	ctx: Context<'_>,
	partial: &'a str
) -> impl Iterator< Item = String > + 'a {
	let srv_id = ctx.guild_id()
		.unwrap();

	let groups: Vec<String> = query_scalar("SELECT group_name from role_groups WHERE server_id = ?")
		.bind(srv_id.get() as i64)
		.fetch_all(&ctx.data().db)
		.await
		.unwrap();

	groups.into_iter()
		.filter(move |f| f.starts_with(partial))
}

#[poise::command(
	guild_only,
	slash_command,
	check = "utils::check_roles",
	required_permissions="MANAGE_ROLES",
	category="Administration",
	ephemeral
)]
async fn delete(
	ctx: Context<'_>,
	#[description = "Name of the role group."]
	#[autocomplete = "autocomplete_groups"]
	group: String
) -> Result<(), Error> {
	let server = if let Some(r) = ctx.guild_id()
		.ok_or_else(|| "Not a Guild")?
		.to_guild_cached(ctx.cache()) {
		r.to_owned()
	} else {
		return Err(serenity::ModelError::GuildNotFound.into())
	};

	// Info Gathering
	let channel: serenity::ChannelId = query_scalar::<_, u64>("SELECT roles_channel FROM server_settings WHERE id = ?;")
		.bind(server.id.get() as i64)
		.fetch_one(&ctx.data().db)
		.await?
		.into();
	let message: serenity::MessageId = query_scalar::<_, u64>("SELECT group_message FROM role_groups WHERE server_id = ? AND group_name = ?;")
		.bind(server.id.get() as i64)
		.bind(&group)
		.fetch_one(&ctx.data().db)
		.await?
		.into();


	// let group_entry = if let Some(rgroup) = query_as::<_, db::Group>(&format!("SELECT group_message FROM role_groups WHERE server_id = ? AND group_name = ?;"))
	// 	.bind(server.id.get() as i64)
	// 	.bind(group)
	// 	.fetch_optional(&ctx.data().db)
	// 	.await? {
	// 	  rgroup
	// } else {
	// 	ctx.send(poise::CreateReply::default()
	// 		.ephemeral(true)
	// 		.embed(templates::state_embed(false, &format!("Role group \"{group}\" does not exist.")))
	// 	).await?;
	// 	return Ok(())
	// };

	// let group_channel = serenity::ChannelId::new(query_as::<_, db::ServerRoles>("SELECT * FROM role_options WHERE srvid = ?;")
	// 	.bind(srv_id.get() as i64)
	// 	.fetch_one(&ctx.data().db)
	// 	.await?
	// 	.channel
	// 	.unwrap_or(srv_id.to_guild_cached(&ctx)
	// 		.unwrap()
	// 		.system_channel_id
	// 		.unwrap_or(ctx.guild()
	// 			.unwrap()
	// 			.default_channel(ctx.framework().bot_id)
	// 			.unwrap()
	// 			.id
	// 		).get() as i64
	// 	) as u64);

	// Deletions
	if ctx.http().delete_message(channel, message, Some(&format!("Deleting Role List \"{group}\""))).await.is_err() {
		ctx.send(poise::CreateReply::default()
			.ephemeral(true)
			.embed(templates::state_embed(false, &format!("Either can't find or can't delete the message for the role group \"{group}\". Entries will be removed from the database, but the message will need to be deleted manually.")))
		).await?;
	}
	// Delete Database Entries
	query("DELETE FROM role_groups WHERE server_id = ? AND group_name = ?;")
		.bind(server.id.get() as i64)
		.bind(&group)
		.execute(&ctx.data().db)
		.await?;
	query("DELETE FROM roles WHERE server_id = ? AND group_name = ?;")
		.bind(server.id.get() as i64)
		.bind(&group)
		.execute(&ctx.data().db)
		.await?;

	// Respond
	ctx.send(poise::CreateReply::default()
		.ephemeral(true)
		.embed(templates::state_embed(true, &format!("Role group \"{group}\" has been deleted.")))
	).await?;
	Ok(())
}

/// Creates a list of roles.
#[poise::command(
	guild_only,
	slash_command,
	check = "utils::check_roles",
	required_permissions="MANAGE_ROLES",
	category="Administration",
	ephemeral
)]
async fn create(
	ctx: Context<'_>,
	#[description = "Name of the role group."]
	group: String
) -> Result<(), Error> {
	let server = if let Some(r) = ctx.guild_id()
		.ok_or_else(|| "Not a Guild")?
		.to_guild_cached(ctx.cache()) {
		r.to_owned()
	} else {
		return Err(serenity::ModelError::GuildNotFound.into())
	};

	// ChannelId from role settings
	let channel = match query_as::<_, db::ServerSettings>("SELECT * FROM server_settings WHERE srvid = ?;")
		.bind(server.id.get() as i64)
		.fetch_one(&ctx.data().db)
		.await?
		.roles_channel {
		Some(chid) => {
			// If exists but can't be posted in just stop and error
			if !utils::can_post(ctx, &chid, ctx.framework().bot_id).await? {
				let name = chid.name(ctx).await.unwrap();
				ctx.send(poise::CreateReply::default()
					.embed(templates::state_embed(false, &format!("Channel \"{name}\" is inaccessable for posting in. Either change the permission overrides or choose a different channel.")))
				).await?;
				return Ok(());
			}
			chid
		},
		None => {
			// If it doesn't exist at all
			ctx.send(poise::CreateReply::default()
				.embed(templates::state_embed(false, "Roles channel is not set and for safety will not be infered. In order to use this command please set the channel with `/setup roles`."))
			).await?;
			return Ok(());
		}
	};

	// Select sorting
	let mut initial_rolelist: Vec<&serenity::Role> = server.roles
		.values()
		.filter(|r| {
			utils::role_filter(r)
		})
		.collect();
	initial_rolelist.sort_by(|a, b| {
		let a_l = a.name.to_lowercase();
		let b_l = b.name.to_lowercase();
		a_l.cmp(&b_l)
	});
	let select_menu: Vec<serenity::CreateSelectMenuOption> = initial_rolelist.into_iter()
		.map(|r| {
			serenity::CreateSelectMenuOption::new(&r.name, r.id.to_string())
		}).collect();

	// The role selection menu.
	let reply = ctx.send(poise::CreateReply::default()
		.embed(serenity::CreateEmbed::new()
			.color(EMBED_STD)
			.description("Please select a set of roles for the group below. Note that roles with permissions to modify the server are not available.")
		).components(vec![
			serenity::CreateActionRow::SelectMenu(serenity::CreateSelectMenu::new("rolelist.new", serenity::CreateSelectMenuKind::String { options: select_menu.clone() })
				.placeholder("Roles")
				.min_values(1)
				.max_values(select_menu.len() as u8))
		])).await?;

	// Await interaction
	let interaction = match reply.message().await?
		.await_component_interaction(ctx)
		.author_id(ctx.author().id)
		.timeout(std::time::Duration::from_secs(300))
		.await {
			Some(i) => {
				i
			},
			None => {
				reply.edit(ctx, poise::CreateReply::default()
					.embed(templates::state_embed(false, "Interaction timed out, please try again."))
				).await?;

				return Ok(())
			}
		};

	// Processing message
	reply.edit(ctx, poise::CreateReply::default()
		.content("")
		.embed(templates::processing_embed())
		.components(Vec::new())
	)
	.await?;

	let selected_roles = if let serenity::ComponentInteractionDataKind::StringSelect { values } = &interaction.data.kind {
		values.into_iter()
		.map(|s| misc::RoleVitals::new(serenity::RoleId::from_str(s)?, &server))
		.collect::<Result<Vec<misc::RoleVitals>, Error>>()?
	} else {
		interaction.create_response(ctx, serenity::CreateInteractionResponse::UpdateMessage(serenity::CreateInteractionResponseMessage::new()
			.embed(templates::state_embed(false, "Somehow recieved wrong interaction, please report this."))
			.components(Vec::new())
		)).await?;
		return Ok(())
	};

	// Send message with roles.
	if let Ok(msg) = channel.send_message(ctx, serenity::CreateMessage::new()
			.embed(templates::rolelist_embed(&group, &selected_roles)?)
			.components(vec![templates::rolelist_components(&group)])
		).await {
		// Add roles if successful
		query("INSERT INTO role_groups (server_id, group_name, group_message) VALUES (?, ?, ?)")
			.bind(server.id.get() as i64)
			.bind(&group)
			.bind(msg.id.get() as i64)
			.execute(&ctx.data().db)
			.await?;
		// Insert roles into database.
		for r in &selected_roles {
			query("INSERT INTO roles (server_id, group_name, role_id) VALUES(?, ?, ?);")
				.bind(server.id.get() as i64)
				.bind(&group)
				.bind(r.id.get() as i64)
				.execute(&ctx.data().db)
				.await?;
		}
		// Notify Success
		reply.edit(ctx, poise::CreateReply::default()
			.embed(templates::state_embed(true, &format!("Rolelist for group \"{group}\" has been created.")))
		).await?;
	} else {
		// Notify Failure
		reply.edit(ctx, poise::CreateReply::default()
			.embed(templates::state_embed(false, &format!("Failed to create role list for group \"{group}\".")))
		).await?;
	}
	Ok(())
}
