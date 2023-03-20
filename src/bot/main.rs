use poise::serenity_prelude as serenity;
use gumdrop::Options;
use sqlx::{
	migrate::MigrateDatabase,
	Sqlite,
	sqlite::SqlitePoolOptions
};
use abby_utils::{Context, Error, Data, Config, db_structs};

mod commands;
mod events;
mod templates;

const EMBED_STD: serenity::utils::Color = serenity::utils::Color::from_rgb(102, 51, 102);
const EMBED_WAIT: serenity::utils::Color = serenity::utils::Color::from_rgb(253, 253, 150);
const EMBED_FAIL: serenity::utils::Color = serenity::utils::Color::from_rgb(178, 34, 34);

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
			commands::setup(),
			commands::bottomify(),
			commands::rolelist()
		],
		prefix_options: poise::PrefixFrameworkOptions {
			prefix: Some("~".into()),
			edit_tracker: Some(poise::EditTracker::for_timespan(std::time::Duration::from_secs(3600))),
			..Default::default()
		},
		event_handler: |ctx, event, framework, data| Box::pin(events::event_handler(ctx, event, framework, data)),
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
						.expect("Failed to listen for closing signal");
					print!("Shutting Down");
					db.close().await;
					sm.lock().await.shutdown_all().await;
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
