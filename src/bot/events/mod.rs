use std::collections::HashSet;
use poise::serenity_prelude as serenity;
use sqlx::{query, query_as};
use crate::{db_structs, Error, Data};

mod message;
mod interaction;

// Private intro function
async fn intro(guild: &serenity::Guild, ctx: &serenity::Context, bot_id: serenity::UserId, db: &sqlx::Pool<sqlx::Sqlite>) -> Result<(), Error> {
	let id = guild.id.as_u64();
	println!("Creating database entry for server {id}.");
	query("INSERT INTO servers (srvid) VALUES(?);")
		.bind(*id as i64)
		.execute(db)
		.await?;
	// If the bot can post in a default channel, do so. Better to disclose the join than not.
	if let Some(chid) = abby_utils::default_bot_channel(ctx, guild.clone(), bot_id).await {
		chid.send_message(ctx, |m| {
			m.content("");
			m.embed(|e|{
				e.title("Hello I'm Abby!");
				e.color(serenity::utils::Color::from_rgb(102, 51, 102));
				e.description("I am general purpose discord bot. To start using my local features on this server, please have an admin run `/setup bot`. For other global commands type /help.");
				e.field("Disclosure", format!("I operate off a database to keep track of settings between servers and reboots. The database consists entirely of booleans and numerical IDs with zero user information or identifying data. If this still concerns you, you can browse the entire implementation at my repository [here]({}).", env!("CARGO_PKG_REPOSITORY")), true)
			})
		}).await?;
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

pub async fn event_handler<'a>(ctx: &serenity::Context, event: &poise::Event<'a>, framework: poise::FrameworkContext<'a, Data, Error>, data: &Data) -> Result<(), Error> {
	match event {
		// Join Server
		poise::Event::GuildCreate { guild, is_new } => {
			if *is_new {
				intro(guild, ctx, framework.bot_id, &data.db).await?;
			} else {
				// If server has been added between logins, run the intro.
				if query("SELECT * FROM servers WHERE srvid = ?;")
					.bind(*guild.id.as_u64() as i64)
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
		poise::Event::GuildDelete { incomplete, full: _ } => {
			kick(*incomplete.id.as_u64(), &data.db).await?;
		},

		// On Login
		poise::Event::Ready { data_about_bot } => {
			println!("{} is connected!", data_about_bot.user.name);
			ctx.set_activity(serenity::Activity::watching("Everything")).await;
			// If servers have been removed, run a purge
			let db_servers: HashSet<u64> = query_as::<_, db_structs::Server>("SELECT * FROM servers;")
				.fetch_all(&data.db)
				.await?
				.into_iter()
				.map(|f| f.srvid as u64)
				.collect();
			let login_servers: HashSet<u64> = data_about_bot.guilds
				.iter()
				.map(|f| *f.id.as_u64())
				.collect();
			for guid in db_servers.difference(&login_servers).collect::<Vec<&u64>>() {
				kick(*guid, &data.db).await?;
			}
		},

		// On Message in Channel
		poise::Event::Message { new_message } => {
			let srv_features = match new_message.guild_id {
				Some(id) => Some(query_as::<_, db_structs::Server>("SELECT * FROM servers WHERE srvid = ?;")
					.bind(*id.as_u64() as i64)
					.fetch_one(&data.db)
					.await?),
				None => None
			}.unwrap_or_default();
			if !new_message.is_own(&ctx.cache) && srv_features.messages {
				message::handler(ctx, new_message, srv_features).await?;
			}
		},

		// Longform Message Interactions
		poise::Event::InteractionCreate { interaction: serenity::Interaction::MessageComponent(m) } => {
			interaction::mci_handler(ctx, data, m).await?;
		}

		// Otherwise Nothing
		_ => ()
	}
	Ok(())
}
