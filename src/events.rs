use std::collections::HashSet;
use color_eyre::{Report, Result};
use poise::serenity_prelude as serenity;
use serenity::{ CreateMessage, CreateEmbedFooter };
use sqlx::{
	query,
	query_scalar
};
use tracing::{
	debug,
	info,
	instrument
};
use crate::{
	commands,
	database,
	templates,
	utils,
	Data
};

mod message;
mod interaction;

#[instrument(skip(db))]
async fn on_join(
	guild_id: serenity::GuildId,
	guild_name: &str,
	db: &sqlx::Pool<sqlx::Sqlite>
) -> Result<()> {
	info!("Joining guild \"{}\" (GuildId {}).", guild_name, guild_id);
	// Do things that a bot would do on server join
	debug!("Creating database entry for guild.");
	{
		let insert = query("INSERT INTO guild_settings (guild_id) VALUES (?);")
			.bind(guild_id.get() as i64)
			.execute(db)
			.await;
		if let Err(e) = insert {
			return Err(e.into())
		}
	}
	Ok(())
}

#[instrument(skip(db))]
// Private on leave function
async fn on_leave(
	guild_id: serenity::GuildId,
	db: &sqlx::Pool<sqlx::Sqlite>
) -> Result<()> {
	info!("Removing guild {guild_id} from database.");
	// Deletion cascades now so this is all we need
	let result = query("DELETE FROM guild_settings WHERE guild_id = ?;")
		.bind(guild_id.get() as i64)
		.execute(db)
		.await;

	match result {
		Ok(_) => Ok(()),
		Err(e) => Err(e.into())
	}
}

#[instrument(skip_all, fields(event))]
pub async fn event_handler<'a>(
	ctx: &serenity::Context,
	event: &serenity::FullEvent,
	framework: poise::FrameworkContext<'a, Data, Report>,
	data: &Data
) -> Result<()> {
	match event {
		// Join Server
		serenity::FullEvent::GuildCreate { guild, is_new } => {
			tracing::Span::current().record("event", "GuildCreate");

			// If server is detected as new. This can fail
			if is_new.is_some_and(|x| x) {
				// Check if server is *actually* new.
				let actually_new = query_scalar::<_, bool>("SELECT NOT EXISTS (SELECT 1 FROM guild_settings WHERE guild_id = ?);")
					.bind(guild.id.get() as i64)
					.fetch_one(&data.db)
					.await;
				match actually_new {
					// New
					Ok(new) if new => {
						on_join(guild.id, &guild.name, &data.db).await?;
						// If the bot can post in a default channel, do so. Better to disclose the join than not.
						if let Some(chid) = utils::default_bot_channel(ctx, guild, framework.bot_id).await? {
							chid.send_message(ctx, CreateMessage::new()
								.embed(
									templates::status::info(Some("Hello I'm AbbyBot"), "In order to use any non-global commands `/setup bot` now. The full list of commands and their details can be found by running `/help`.")
									.field("Privacy", format!("In order to save per-server settings a database is used to store data based on its id number. If you have concerns the full implementation is viewable [here]({}).", env!("CARGO_PKG_REPOSITORY")), true)
									.footer(CreateEmbedFooter::new("This is a one time message and should not be sent again."))
								)
							).await?;
						}
					},
					// Error
					Err(e) => return Err(e.into()),
					// Not New
					_ => ()
				}
			}
		},

		// Kicked from Server
		serenity::FullEvent::GuildDelete { incomplete, .. } => {
			tracing::Span::current().record("event", "GuildDelete");

			// Should only be leaving servers when kicked/banned
			if !incomplete.unavailable {
				on_leave(incomplete.id, &data.db).await?;
			}
		},

		// On Login
		serenity::FullEvent::Ready { data_about_bot } => {
			tracing::Span::current().record("event", "Ready");

			#[cfg(feature = "systemd")]
			sd_notify::notify(false, &[sd_notify::NotifyState::Ready]);

			info!("{} is connected!", data_about_bot.user.name);
			// Register global commands
			poise::builtins::register_globally(ctx, &commands::global_commands()).await?;
			// Set status because memes
			let status = serenity::ActivityData {
				name: "Everything".to_string(),
				kind: serenity::ActivityType::Watching,
				state: None,
				url: None
			};
			ctx.set_activity(Some(status));
			// If servers have been removed, run a purge
			let db_guilds: HashSet<u64> = query_scalar("SELECT guild_id FROM guild_settings;")
				.fetch_all(&data.db)
				.await?
				.into_iter()
				.collect();
			for unfinished_guild in &data_about_bot.guilds {
				if !unfinished_guild.unavailable {
					// Unavailable being false means removal while offline
					on_leave(unfinished_guild.id, &data.db).await?;
				} else if !db_guilds.contains(&unfinished_guild.id.get()) {
					// Servers that arn't in the database have been joined while offline
					// Can't send a join message from here due to incomplete guild data
					let partial = unfinished_guild.id.to_partial_guild(ctx).await.unwrap();
					on_join(partial.id, &partial.name, &data.db).await?;
				}
			}
		},

		// On Message in Channel
		serenity::FullEvent::Message { new_message } => {
			tracing::Span::current().record("event", "Message");
			let guild_settings = if let Some(id) = new_message.guild_id {
				database::GuildSettings::from_query(id, &data.db).await?
			} else {
				database::GuildSettings::private_default()
			};
			if (new_message.author.id != framework.bot_id) && guild_settings.messages {
				message::handler(ctx, new_message, guild_settings).await?;
			}
		},

		// Longform Message Interactions
		serenity::FullEvent::InteractionCreate { interaction: serenity::Interaction::Component(m) } => {
			tracing::Span::current().record("event", "InteractionCreate");
			interaction::mci_handler(ctx, data, m).await?;
		}

		// Otherwise Nothing
		_ => {}
	}
	Ok(())
}
