use poise::serenity_prelude as serenity;
use sqlx::{
	query,
	query_as
};
use crate::{
	structs::db,
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
) -> Vec<String> {
	let srv_id = ctx.guild_id().unwrap();
	let groups = query_as::<_, db::RoleGroup>(&format!("SELECT * FROM rgroups_{srv_id};"))
		.fetch_all(&ctx.data().db)
		.await
		.unwrap();
	groups.into_iter()
		.filter_map(|f| {
			if f.name.starts_with(partial) {
				return Some(f.name)
			}
			None
		}).collect::<Vec<String>>()
}

#[poise::command(
	guild_only,
	slash_command,
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
	let srv_id = ctx.guild_id().unwrap();
	let srv_features = query_as::<_, db::Server>("SELECT * FROM servers WHERE srvid = ?;")
		.bind(srv_id.get() as i64)
		.fetch_one(&ctx.data().db)
		.await?;
	if !srv_features.roles {
		utils::feature_not_enabled(ctx).await?;
		return Ok(())
	}

	// Info Gathering
	let group_entry = if let Some(rgroup) = query_as::<_, db::RoleGroup>(&format!("SELECT * FROM rgroups_{srv_id} WHERE name = ?;"))
		.bind(&group)
		.fetch_optional(&ctx.data().db)
		.await? {
		  rgroup
	} else {
		ctx.send(poise::CreateReply::default()
			.ephemeral(true)
			.embed(templates::state_embed(false, &format!("Role group \"{group}\" does not exist.")))
		).await?;
		return Ok(())
	};

	let group_channel = serenity::ChannelId::new(query_as::<_, db::ServerRoles>("SELECT * FROM role_options WHERE srvid = ?;")
		.bind(srv_id.get() as i64)
		.fetch_one(&ctx.data().db)
		.await?
		.channel
		.unwrap_or(srv_id.to_guild_cached(&ctx)
			.unwrap()
			.system_channel_id
			.unwrap_or(ctx.guild()
				.unwrap()
				.default_channel(ctx.framework().bot_id)
				.unwrap()
				.id
			).get() as i64
		) as u64);

	// Deletions
	if ctx.http().delete_message(group_channel, serenity::MessageId::new(group_entry.msg as u64), Some(&format!("Deleting Role List {}", group_entry.name))).await.is_err() {
		ctx.send(poise::CreateReply::default()
			.ephemeral(true)
			.embed(templates::state_embed(false, &format!("Either can't find or can't delete the message for the role group \"{group}\". Entries will be removed from the database, but the message will need to be deleted manually.")))
		).await?;
	}
	query(&format!("DELETE FROM rgroups_{srv_id} WHERE name = ?;"))
		.bind(&group)
		.execute(&ctx.data().db)
		.await?;
	query(&format!("DELETE FROM roles_{srv_id} WHERE grp = ?;"))
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
	required_permissions="MANAGE_ROLES",
	category="Administration",
	ephemeral
)]
async fn create(
	ctx: Context<'_>,
	#[description = "Name of the role group."]
	group: String
) -> Result<(), Error> {
	let srv_id = ctx.guild_id().unwrap().get();
	let srv_features = query_as::<_, db::Server>("SELECT * FROM servers WHERE srvid = ?;")
		.bind(srv_id as i64)
		.fetch_one(&ctx.data().db)
		.await?;
	// If the feature doesn't exist, quit.
	if !srv_features.roles {
		utils::feature_not_enabled(ctx).await?;
		return Ok(())
	}

	// ChannelId from role settings
	let channel = match query_as::<_, db::ServerRoles>("SELECT * FROM role_options WHERE srvid = ?;")
		.bind(srv_id as i64)
		.fetch_one(&ctx.data().db)
		.await?
		.channel
		.map(|id| serenity::ChannelId::from(id as u64)) {
		Some(chid) => {
			// If exists but can't be posted in just stop and error
			if !utils::can_post(ctx, &chid, ctx.framework().bot_id).await {
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
	let mut initial_rolelist: Vec<serenity::Role> = ctx.guild()
		.unwrap()
		.roles
		.clone()
		.into_values()
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

	let selected = match interaction.data.kind {
		serenity::ComponentInteractionDataKind::StringSelect { values } => values,
		_ => {
			println!("Nope");
			return Ok(())
		}
	};

	// Insert roles into database.
	for rid in selected {
		let users = ctx.guild_id()
			.unwrap()
			.members(ctx, None, None)
			.await?
			.into_iter()
			.filter_map(|f| {
				if f.roles.contains(&serenity::RoleId::new(rid.parse::<u64>().unwrap())) {
					return Some(f)
				}
				None
			}).count();
		query(&format!("INSERT INTO roles_{srv_id} (id, grp, users) VALUES(?, ?, ?);"))
			.bind(rid)
			.bind(&group)
			.bind(users as i64)
			.execute(&ctx.data().db)
			.await?;
	}

	// Fetch new rolelist
	let rolelist = query_as::<_, db::RoleEntry>(&format!("SELECT * FROM roles_{srv_id} WHERE grp = ?;"))
		.bind(&group)
		.fetch_all(&ctx.data().db)
		.await?;

	// Generate list with the channel stored for the check
	let msg = channel.send_message(ctx, serenity::CreateMessage::new()
		.embed(templates::rolelist_embed(ctx.serenity_context(), &group, rolelist, ctx.guild_id().unwrap()))
		.components(vec![templates::rolelist_components(&group)])
	).await?;

	// Add msg to group table
	query(&format!("INSERT INTO rgroups_{srv_id} (name, msg) VALUES (?, ?)"))
		.bind(&group)
		.bind(msg.id.get() as i64)
		.execute(&ctx.data().db)
		.await?;

	// Notify Success
	reply.edit(ctx, poise::CreateReply::default()
		.embed(templates::state_embed(true, &format!("Rolelist for group {group} has been created.")))
	).await?;
	Ok(())
}
