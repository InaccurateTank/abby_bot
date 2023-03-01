use poise::serenity_prelude as serenity;
use gumdrop::Options;
use sqlx::{
	migrate::MigrateDatabase,
	Sqlite,
	sqlite::SqlitePoolOptions,
	query, query_as
};
use abby_utils::{
	Context,
	Error,
	Data,
	Config,
	db_structs
};

mod commands;
mod events;

#[derive(Debug, Options)]
struct Opts {
	help: bool,
	#[options(help = "Set data folder location.", default = "data/")]
	data: String
}

#[tokio::main]
async fn main() -> Result<(), Error> {
	// Opts
	let opts = Opts::parse_args_default_or_exit();

	// Data Folder From Opts
	let data_folder = if !opts.data.ends_with('/') {
		abby_utils::concat(&opts.data, "/")
	} else {
		opts.data
	};

	// Config
	let config = Config::new(&data_folder)?;

	// DB
	let db_url = format!("sqlite:{data_folder}sqlite.db");
	if !Sqlite::database_exists(&db_url).await.unwrap_or(false) {
		println!("Database absent in '{data_folder}', creating...");
		match Sqlite::create_database(&db_url).await {
			Ok(_) => println!("DB Creation Success!"),
			Err(error) => panic!("error: {error}")
		}
	}
	println!("Connecting to database...");
	let pool = SqlitePoolOptions::new()
		.max_connections(5)
		.connect(&db_url).await?;
	sqlx::migrate!("./migrations")
		.run(&pool)
		.await?;
	println!("Database Connection established!");

	// Bot Options
	let options = poise::FrameworkOptions {
		commands: vec![
			commands::help(),
			commands::about(),
			commands::register(),
			commands::bottomify(),
			// commands::roles()
			commands::setup()
		],
		prefix_options: poise::PrefixFrameworkOptions {
			prefix: Some("~".into()),
			edit_tracker: Some(poise::EditTracker::for_timespan(std::time::Duration::from_secs(3600))),
			..Default::default()
		},
		event_handler: |ctx, event, _framework, data| {
			Box::pin(async move {

				match event {
					// Join Server
					poise::Event::GuildCreate { guild, is_new } => {
						if let Some(id) = guild.system_channel_id {
							if *is_new {
								id.send_message(ctx, |m| {
									m.content("")
									.embed(|e|{
										e.title("Hello I'm Abby!")
											.color(serenity::utils::Color::new(663366))
											.description("I am general purpose discord bot. To start using my local features on this server, please have an admin run `/setup bot`. For other global commands type /help.")
											.field("Disclosure", format!("I operate off a database to keep track of settings between servers and reboots. The database consists entirely of booleans and numerical IDs with zero user information or identifying data. If this still concerns you, you can browse the entire implementation at my repository [here]({}).", env!("CARGO_PKG_REPOSITORY")), true)
									})
								}).await?;
								let id = guild.id.as_u64();
								println!("Creating database entry for server {id}.");
								query("INSERT INTO servers (srvid) VALUES(?);")
									.bind(*id as i64)
									.execute(&data.db)
									.await?;
							}
						};
					},

					// Kicked from Server
					poise::Event::GuildDelete { incomplete, full: _ } => {
						let id = incomplete.id.as_u64();
						println!("Deleting server {id} from database.");
						query("DELETE FROM servers WHERE srvid = ?;")
							.bind(*id as i64)
							.execute(&data.db)
							.await?;
						query("DELETE FROM roles WHERE srvid = ?;")
							.bind(*id as i64)
							.execute(&data.db)
							.await?;
						query(&format!("DROP TABLE IF EXISTS roles_{id};"))
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
								.await
								.unwrap()),
							None => None
						}.unwrap_or_default();
						if !new_message.is_own(&ctx.cache) && srv_features.messages {
							events::message::handler(ctx, new_message, srv_features).await?;
						}
					},

					// On Interaction
					poise::Event::InteractionCreate { interaction } => {
						match interaction {
							serenity::Interaction::MessageComponent(m) => {
								let inter_id = &m.data.custom_id;
								if !inter_id.starts_with("register") && !inter_id.starts_with("unregister") && !inter_id.starts_with("setup") {
									events::interaction::mci_handler(ctx, m).await?;
								}
							}
							_ => {}
						}


						// if let Some(mci) = &interaction.clone().message_component() {
						// 	let inter_id = &*mci.data.custom_id;
						// 	if !inter_id.starts_with("register") && !inter_id.starts_with("unregister") && !inter_id.starts_with("setup") {
						// 		events::interaction::handler(ctx, interaction).await?;
						// 	}

						// 	// if !inter_id.starts_with("register") && !inter_id.starts_with("unregister") && !inter_id.starts_with("setup") {
						// 	// 	events::roles_click(ctx, mci).await?;
						// 	// }
						// }
					}

					// Otherwise Nothing
					_ => ()
				}
				Ok(())
			})
		},
		..Default::default()
	};
	let intents = serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::GUILD_MESSAGES | serenity::GatewayIntents::MESSAGE_CONTENT;

	// Bot Setup
	let framework = poise::Framework::builder()
		.options(options)
		.token(&*config.token)
		.intents(intents)
		.setup(move |_ctx, _ready, framework| {
			Box::pin(async move {
				let sm = framework.shard_manager().clone();
				let db = pool.clone();
				tokio::spawn(async move {
					tokio::signal::ctrl_c()
						.await
						.expect("Failed to listen for Ctrl+C");
					print!("Shutting Down");
					sm.lock().await.shutdown_all().await;
					db.close().await;
				});
				Ok(Data {
					db: pool
				})
			})
		});

	// Start Bot
	if let Err(why) = framework.run().await {
		println!("Client error: {why:?}");
	}
	Ok(())
}
