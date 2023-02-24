use poise::serenity_prelude as serenity;
use gumdrop::Options;
use sqlx::{migrate::MigrateDatabase, Sqlite, sqlite::SqlitePoolOptions};
use abby_utils::{Context, Error, Data, concat, Config};

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

	// Folder from opts
	let data_folder = if !opts.data.ends_with("/") {
		concat(&opts.data, "/")
	} else {
		opts.data
	};

	// Config
	let config = Config::new(&data_folder)?;

	// DB
	let db_url = format!("sqlite:{}sqlite.db", &data_folder);
	if !Sqlite::database_exists(&db_url).await.unwrap_or(false) {
		println!("Database absent in '{}', creating...", data_folder);
		match Sqlite::create_database(&db_url).await {
			Ok(_) => println!("DB Creation Success!"),
			Err(error) => panic!("error: {}", error)
		}
	}
	println!("Connecting to database...");
	let pool = SqlitePoolOptions::new()
		.max_connections(5)
		.connect(&db_url).await?;
	println!("Database Connection established!");

	// Bot Start
	let options = poise::FrameworkOptions {
		commands: vec![
			commands::help(),
			commands::about(),
			commands::register(),
			commands::bottomify(),
			commands::roles()
		],
		prefix_options: poise::PrefixFrameworkOptions {
			prefix: Some("~".into()),
			edit_tracker: Some(poise::EditTracker::for_timespan(std::time::Duration::from_secs(3600))),
			..Default::default()
		},
		event_handler: |ctx, event, _framework, _data| {
			Box::pin(async move {
				match event {
					poise::Event::Ready { data_about_bot } => {
						println!("{} is connected!", data_about_bot.user.name);
						ctx.set_activity(serenity::Activity::watching("Everything")).await;
						// for guild in &data_about_bot.guilds {
						// 	if guild.unavailable {
						// 		poise::builtins::register_in_guild(&ctx, &framework.options().commands, guild.id).await?;
						// 		println!("Registered commands in guild {}", guild.id.name(&ctx).unwrap());
						// 	}
						// }
					},
					poise::Event::Message { new_message } => {
						if !new_message.is_own(&ctx.cache) {
							let m = new_message.content.as_str();
							events::borger(ctx, new_message, &m).await?;
							events::v(ctx, new_message, &m).await?;
						}
					},
					poise::Event::InteractionCreate { interaction } => {
						if let Some(mci) = &interaction.clone().message_component() {
							if !mci.data.custom_id.starts_with("register") && !mci.data.custom_id.starts_with("unregister") {
								events::roles_click(ctx, mci).await?;
							}
						}
					}
					_ => ()
				}
				Ok(())
			})
		},
		..Default::default()
	};
	let intents = serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::GUILD_MESSAGES | serenity::GatewayIntents::MESSAGE_CONTENT;

	let framework = poise::Framework::builder()
		.options(options)
		.token(&*config.token)
		.intents(intents)
		.setup(move |_ctx, _ready, framework| {
			Box::pin(async move {
				let sm = framework.shard_manager().clone();
				tokio::spawn(async move {
					tokio::signal::ctrl_c()
						.await
						.expect("Failed to listen for Ctrl+C");
					print!("Shutting Down");
					sm.lock().await.shutdown_all().await;
				});
				Ok(Data {
					db: pool
				})
			})
		});

	if let Err(why) = framework.run().await {
		println!("Client error: {:?}", why);
	}
	Ok(())
}
