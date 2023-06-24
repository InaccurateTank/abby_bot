use poise::serenity_prelude as serenity;
use crate::{db_structs,Error, Data};
use sqlx::{query, query_as};

mod message;
mod interaction;

pub async fn event_handler<'a>(ctx: &serenity::Context, event: &poise::Event<'a>, framework: poise::FrameworkContext<'a, Data, Error>, data: &Data) -> Result<(), Error> {
	match event {
		// Join Server
		poise::Event::GuildCreate { guild, is_new } => {
			if *is_new {
				let id = guild.id.as_u64();
				println!("Creating database entry for server {id}.");
				query("INSERT INTO servers (srvid) VALUES(?);")
					.bind(*id as i64)
					.execute(&data.db)
					.await?;
				// If the bot can post in a default channel, do so. Better to disclose the join than not.
				if let Some(chid) = abby_utils::default_bot_channel(ctx, guild.clone(), framework.bot_id).await {
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
			} else {
				// Re-register commands on login to keep up with updates
				poise::builtins::register_in_guild(ctx, framework.options().commands.as_slice(), guild.id).await?;
			}
		},

		// Kicked from Server
		poise::Event::GuildDelete { incomplete, full: _ } => {
			let id = incomplete.id.as_u64();
			println!("Deleting server {id} from database.");
			query("DELETE FROM servers WHERE srvid = ?;")
				.bind(*id as i64)
				.execute(&data.db)
				.await?;

			// Can't purge the messages so just remove the roles.
			query("DELETE FROM role_options WHERE srvid = ?;")
				.bind(*id as i64)
				.execute(&data.db)
				.await?;
			query(&format!("DROP TABLE IF EXISTS roles_{id};"))
				.execute(&data.db)
				.await?;
			query(&format!("DROP TABLE IF EXISTS rgroups_{id};"))
				.execute(&data.db)
				.await?;
		},

		// On Login
		poise::Event::Ready { data_about_bot } => {
			println!("{} is connected!", data_about_bot.user.name);
			ctx.set_activity(serenity::Activity::watching("Everything")).await;
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
