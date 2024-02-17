use std::collections::HashSet;
use poise::serenity_prelude as serenity;
use serenity::{ CreateMessage, CreateEmbed };
use sqlx::{query, query_as};
use crate::{ EMBED_STD, Error, Data };
use crate::structs::db;
use crate::utils;

mod message;
mod interaction;

// Private intro function
async fn intro(guild: &serenity::Guild, ctx: &serenity::Context, bot_id: serenity::UserId, db: &sqlx::Pool<sqlx::Sqlite>) -> Result<(), Error> {
	let id = guild.id.get();
	println!("Creating database entry for server {id}.");
	query("INSERT INTO servers (srvid) VALUES(?);")
		.bind(id as i64)
		.execute(db)
		.await?;
	// If the bot can post in a default channel, do so. Better to disclose the join than not.
	if let Some(chid) = utils::default_bot_channel(ctx, guild.clone(), bot_id).await {
		chid.send_message(ctx, CreateMessage::new()
			.embed(CreateEmbed::new()
				.title("Hello I'm Abby!")
				.color(EMBED_STD)
				.description("I am general purpose discord bot. To start using my local features on this server, please have an admin run `/setup bot`. For other global commands type /help.")
				.field("Disclosure", format!("I operate off a database to keep track of settings between servers and reboots. The database consists entirely of booleans and numerical IDs with zero user information or identifying data. If this still concerns you, you can browse the entire implementation at my repository [here]({}).", env!("CARGO_PKG_REPOSITORY")), true)
			)
		).await?;
	}
	Ok(())
}

// Private kick function
async fn kick(guid: u64, db: &sqlx::Pool<sqlx::Sqlite>) -> Result<(), Error> {
	println!("Deleting server {guid} from database.");
	query("DELETE FROM servers WHERE srvid = ?;")
		.bind(guid as i64)
		.execute(db)
		.await?;

	// Can't purge the messages so just remove the roles.
	query("DELETE FROM role_options WHERE srvid = ?;")
		.bind(guid as i64)
		.execute(db)
		.await?;
	query(&format!("DROP TABLE IF EXISTS roles_{guid};"))
		.execute(db)
		.await?;
	query(&format!("DROP TABLE IF EXISTS rgroups_{guid};"))
		.execute(db)
		.await?;
	Ok(())
}

pub async fn event_handler<'a>(ctx: &serenity::Context, event: &serenity::FullEvent, framework: poise::FrameworkContext<'a, Data, Error>, data: &Data) -> Result<(), Error> {
	match event {
		// Join Server
		serenity::FullEvent::GuildCreate { guild, is_new } => {
			if is_new.is_some_and(|x| x) {
				intro(guild, ctx, framework.bot_id, &data.db).await?;
			} else {
				// If server has been added between logins, run the intro.
				if query("SELECT * FROM servers WHERE srvid = ?;")
					.bind(guild.id.get() as i64)
					.fetch_one(&data.db)
					.await
					.is_err() {
					intro(guild, ctx, framework.bot_id, &data.db).await?;
				}
			}
			// Auto-register commands to keep up with updates
			poise::builtins::register_in_guild(ctx, framework.options().commands.as_slice(), guild.id).await?;
		},

		// Kicked from Server
		serenity::FullEvent::GuildDelete { incomplete, full: _ } => {
			kick(incomplete.id.get(), &data.db).await?;
		},

		// On Login
		serenity::FullEvent::Ready { data_about_bot } => {
			println!("{} is connected!", data_about_bot.user.name);
			let status = serenity::ActivityData {
				name: "Everything".to_string(),
				kind: serenity::ActivityType::Watching,
				state: None,
				url: None
			};
			ctx.set_activity(Some(status));
			// If servers have been removed, run a purge
			let db_servers: HashSet<u64> = query_as::<_, db::Server>("SELECT * FROM servers;")
				.fetch_all(&data.db)
				.await?
				.into_iter()
				.map(|f| f.srvid as u64)
				.collect();
			let login_servers: HashSet<u64> = data_about_bot.guilds
				.iter()
				.map(|f| f.id.get())
				.collect();
			for guid in db_servers.difference(&login_servers).collect::<Vec<&u64>>() {
				kick(*guid, &data.db).await?;
			}
		},

		// On Message in Channel
		serenity::FullEvent::Message { new_message } => {
			let srv_features = match new_message.guild_id {
				Some(id) => Some(query_as::<_, db::Server>("SELECT * FROM servers WHERE srvid = ?;")
					.bind(id.get() as i64)
					.fetch_one(&data.db)
					.await?),
				None => None
			}.unwrap_or_default();
			if !new_message.is_own(&ctx.cache) && srv_features.messages {
				message::handler(ctx, new_message, srv_features).await?;
			}
		},

		// Longform Message Interactions
		serenity::FullEvent::InteractionCreate { interaction: serenity::Interaction::Component(m) } => {
			interaction::mci_handler(ctx, data, m).await?;
		}

		// Otherwise Nothing
		_ => {}
	}
	Ok(())
}
