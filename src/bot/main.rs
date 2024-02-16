use poise::serenity_prelude as serenity;
use gumdrop::Options;
use sqlx::{
	migrate::MigrateDatabase,
	Sqlite,
	sqlite::SqlitePoolOptions
};

mod commands;
mod events;
mod structs;
mod templates;
mod utils;

use structs::Config;

const EMBED_STD: serenity::Color = serenity::Color::from_rgb(102, 51, 102);
const EMBED_WAIT: serenity::Color = serenity::Color::from_rgb(253, 253, 150);
const EMBED_FAIL: serenity::Color = serenity::Color::from_rgb(178, 34, 34);

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;

pub struct Data {
	pub db: sqlx::Pool<sqlx::Sqlite>
}

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
		utils::concat(&opts.data, "/")
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
			commands::rolelist(),
			// commands::snap(),
			// commands::rewind()
		],
		prefix_options: poise::PrefixFrameworkOptions {
			prefix: Some("~".into()),
			edit_tracker: Some(poise::EditTracker::for_timespan(std::time::Duration::from_secs(3600)).into()),
			..Default::default()
		},
		event_handler: |ctx, event, framework, data| Box::pin(events::event_handler(ctx, event, framework, data)),
		..Default::default()
	};
	let intents = serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::GUILD_MESSAGES | serenity::GatewayIntents::MESSAGE_CONTENT;

	// Bot Setup
	let framework = poise::Framework::builder()
		.options(options)
		.setup(move |_ctx, _ready, framework| {
			Box::pin(async move {
				let sm = framework.shard_manager().clone();
				let db = pool.clone();
				tokio::spawn(async move {
					#[cfg(target_os = "linux")]
					{
						use tokio::signal::unix::{signal, SignalKind};
						let mut sigint = signal(SignalKind::interrupt()).unwrap();
						let mut sigterm = signal(SignalKind::terminate()).unwrap();
						tokio::select! {
							_ = sigint.recv() => {},
							_ = sigterm.recv() => {}
						}
					}
					#[cfg(target_os = "windows")]
					{
						use tokio::signal::windows::{ctrl_c, ctrl_close};
						let mut c = ctrl_c().unwrap();
						let mut close = ctrl_close().unwrap();
						tokio::select! {
							_ = c.recv() => {},
							_ = close.recv() => {}
						}
					}
					print!("Shutting Down");
					db.close().await;
					sm.shutdown_all().await;
				});
				Ok(Data {
					db: pool
				})
			})
		})
		.build();

	let mut client = match serenity::ClientBuilder::new(config.token, intents)
		.framework(framework)
		.await {
		Ok(c) => c,
		Err(e) => {
			println!("Error assembling client: {e:?}");
			return Ok(())
		}
	};

	if let Err(why) = client.start().await {
		println!("Client error: {why:?}");
	}
	Ok(())
}
