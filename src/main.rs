use std::str::FromStr;
use poise::serenity_prelude as serenity;
use gumdrop::Options;
use sqlx::{
	migrate::MigrateDatabase,
	Sqlite,
	sqlite::{SqliteConnectOptions, SqlitePoolOptions}
};
use tracing::{
	debug,
	info,
	warn,
	error,
	level_filters::LevelFilter
};

mod commands;
mod database;
mod error;
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
	data: String,
	#[options(count, help = "Set data folder location.")]
	verbose: u8
}

#[tokio::main]
async fn main() -> Result<(), Error> {
	// Opts
	let opts = Opts::parse_args_default_or_exit();

	let verbosity = match opts.verbose {
		0 => LevelFilter::INFO,
		1 => LevelFilter::DEBUG,
		_ => LevelFilter::TRACE
	};
	tracing_subscriber::fmt()
		.with_max_level(verbosity)
		.without_time()
		.init();

	// Data Folder From Opts
	let data_folder = if !opts.data.ends_with('/') {
		utils::concat(&opts.data, "/")
	} else {
		opts.data
	};

	debug!("Loading configuration.");
	// Config
	let config = Config::new(&data_folder)?;

	// DB
	let db_url = format!("sqlite:{data_folder}sqlite.db");
	// Check if DB exists
	if !Sqlite::database_exists(&db_url).await.unwrap_or(false) {
		warn!("Database absent in '{data_folder}', creating new one.");
		match Sqlite::create_database(&db_url).await {
			Ok(_) => debug!("Database creation successful."),
			Err(error) => panic!("error: {error}")
		}
	}
	info!("Connecting to database.");
	// Set DB options
	let db_opts = SqliteConnectOptions::from_str(&db_url)?
		.foreign_keys(true);
	// Connect and migrate
	let pool = SqlitePoolOptions::new()
		.max_connections(5)
		.connect_with(db_opts).await?;
	sqlx::migrate!("./migrations")
		.run(&pool)
		.await?;
	debug!("Database Connection successfully established.");

	// Bot Options
	let options = poise::FrameworkOptions {
		commands: vec![
			commands::help(),
			commands::about(),
			commands::register(),
			commands::setup(),
			commands::bottomify(),
			commands::rolelist(),
			commands::snap(),
			commands::rewind()
		],
		prefix_options: poise::PrefixFrameworkOptions {
			prefix: Some("~".into()),
			edit_tracker: Some(poise::EditTracker::for_timespan(std::time::Duration::from_secs(3600)).into()),
			..Default::default()
		},
		event_handler: |ctx, event, framework, data| Box::pin(events::event_handler(ctx, event, framework, data)),
		pre_command: |ctx| {
			Box::pin(async move {
				debug!("{} invoked {}.", ctx.author().name, ctx.invocation_string());
			})
		},
		post_command: |ctx| {
			Box::pin(async move {
				debug!("Invocation {} from {} finished successfully.", ctx.invocation_string(), ctx.author().name);
			})
		},
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
			error!("Error with Discord client: {e:?}");
			return Ok(())
		}
	};

	if let Err(why) = client.start().await {
		error!("Client error: {why:?}");
	}
	Ok(())
}
