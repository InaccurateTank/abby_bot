use std::collections::HashSet;
use poise::serenity_prelude as serenity;
use serenity::{ CreateMessage, CreateEmbed };
use sqlx::{query, query_as};
use crate::{ EMBED_STD, Error, Data };
use crate::structs::db;
use crate::utils;

mod message;
mod interaction;

// Private on leave function
async fn on_leave(guid: u64, db: &sqlx::Pool<sqlx::Sqlite>) -> Result<(), Error> {
	println!("Deleting server {guid} from database.");
	// Deletion cascades now so this is all we need
	query("DELETE FROM server_settings WHERE id = ?;")
		.bind(guid as i64)
		.execute(db)
		.await?;
	Ok(())
}

pub async fn event_handler<'a>(ctx: &serenity::Context, event: &serenity::FullEvent, framework: poise::FrameworkContext<'a, Data, Error>, data: &Data) -> Result<(), Error> {
	match event {
		// Join Server
		serenity::FullEvent::GuildCreate { guild, is_new } => {
			// If server is detected as new. This can fail
			if is_new.is_some_and(|x| x) {
				// Check if server is *actually* new.
				if query_scalar::<_, bool>("SELECT NOT EXISTS (SELECT 1 FROM server_settings WHERE id = 1);")
					.bind(guild.id.get() as i64)
					.fetch_one(&data.db)
					.await? {
					// Do things that a bot would do on server join
					let id = guild.id.get();
					// TODO: Log the console errors
					println!("Creating database entry for server {} (GuildId {}).", guild.name, id);
					query("INSERT INTO server_settings (id) VALUES(?);")
						.bind(id as i64)
						.execute(&data.db)
						.await?;
					// If the bot can post in a default channel, do so. Better to disclose the join than not.
					// TODO: Change intro message
					if let Some(chid) = utils::default_bot_channel(ctx, guild.to_owned(), framework.bot_id).await? {
						chid.send_message(ctx, CreateMessage::new()
							.embed(CreateEmbed::new()
								.title("Hello I'm Abby!")
								.color(EMBED_STD)
								.description("I am general purpose discord bot. To start using my local features on this server, please have an admin run `/setup bot`. For other global commands type /help.")
								.field("Disclosure", format!("I operate off a database to keep track of settings between servers and reboots. The database consists entirely of booleans and numerical IDs with zero user information or identifying data. If this still concerns you, you can browse the entire implementation at my repository [here]({}).", env!("CARGO_PKG_REPOSITORY")), true)
							)
						).await?;
					}
					// Register commands in new server
					// TODO: Change to register commangs via settings
					poise::builtins::register_in_guild(ctx, framework.options().commands.as_slice(), guild.id).await?;
				}
			}
		},

		// Kicked from Server
		serenity::FullEvent::GuildDelete { incomplete, full: _ } => {
			// Should only be leaving servers when kicked/banned
			if !incomplete.unavailable {
				on_leave(incomplete.id.get(), &data.db).await?;
			}
		},

		// On Login
		serenity::FullEvent::Ready { data_about_bot } => {
			// TODO: Log
			println!("{} is connected!", data_about_bot.user.name);
			// Set status because memes
			let status = serenity::ActivityData {
				name: "Everything".to_string(),
				kind: serenity::ActivityType::Watching,
				state: None,
				url: None
			};
			ctx.set_activity(Some(status));
			// If servers have been removed, run a purge
			let db_servers: HashSet<u64> = query_scalar("SELECT id FROM server_settings;")
				.fetch_all(&data.db)
				.await?
				.into_iter()
				.collect();
			let login_servers: HashSet<u64> = data_about_bot.guilds
				.iter()
				.map(|f| f.id.get())
				.collect();
			for guid in db_servers.difference(&login_servers) {
				on_leave(*guid, &data.db).await?;
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
			if (new_message.author.id != framework.bot_id) && srv_features.messages {
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
