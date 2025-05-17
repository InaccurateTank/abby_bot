// use std::str::FromStr;
use color_eyre::{Report, Result};
use poise::serenity_prelude as serenity;
// use gumdrop::Options;
use sqlx::{
	// migrate::MigrateDatabase,
	// Sqlite,
	sqlite::{SqliteConnectOptions, SqlitePoolOptions}
};
use tracing::{
	instrument,
	debug, error, info, warn
};

mod checks;
mod commands;
mod database;
mod error;
mod events;
mod structs;
mod templates;
mod utils;
mod colors {
	//! Simple module to store color constants.
	use poise::serenity_prelude as serenity;

	pub const INFO: serenity::Color = serenity::Color::from_rgb(102, 51, 102);
	pub const WARN: serenity::Color = serenity::Color::from_rgb(253, 253, 150);
	pub const ERROR: serenity::Color = serenity::Color::from_rgb(178, 34, 34);
}

// pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Report>;

#[derive(Debug)]
pub struct Data {
	pub db: sqlx::Pool<sqlx::Sqlite>
}

#[instrument]
#[tokio::main]
async fn main() -> Result<()> {
	color_eyre::install()?;

	// Options
	let opts = structs::Opts::parse();

	// Installing tracing subscriber
	install_tracing(&opts);

	// Data Folder From Opts
	// let data_folder = opts.data_dir.to_string_lossy();

	// let data_folder = if !opts.data_dir.ends_with('/') {
	// 	concat!(&opts.data_dir, "/")
	// } else {
	// 	opts.data_dir
	// };
	// let config_folder = if let Some(dir) = opts.config_dir {
	// 	dir
	// } else {
	// 	concat!(&data_folder, "/config")
	// };

	// let config_folder = if !opts.config_dir.ends_with('/') {
	// 	concat!(&opts.config_dir, "/")
	// } else {
	// 	opts.config_dir
	// };

	// Config
	info!("Loading configuration {}", opts.configuration.to_string_lossy());
	let config = structs::Config::load(opts.configuration)?;

	// // DB
	// let db_url = format!("sqlite://{}sqlite.db", opts.data_dir.to_string_lossy());
	// // Check if DB exists
	// if !Sqlite::database_exists(&db_url).await.unwrap_or(false) {
	// 	warn!("Database absent in '{}', creating new one.", opts.data_dir.to_string_lossy());
	// 	// std::fs::exists(path);
	// 	match Sqlite::create_database(&db_url).await {
	// 		Ok(_) => debug!("Database creation successful."),
	// 		Err(error) => panic!("error: {error}")
	// 	}
	// }
	// info!("Connecting to database.");


	// Set DB options
	// let db_opts = SqliteConnectOptions::new()
	// 	.filename(db_url)
	// 	.create_if_missing(true)
	// 	.foreign_keys(true);
	// let db_opts = SqliteConnectOptions::from_str(&db_url)?
	// 	.foreign_keys(true);
	// Connect and migrate

	// Data Directory
	if !opts.data_dir.try_exists()? {
		warn!("Attempting to initialize missing data directory {}", opts.data_dir.to_string_lossy());
		std::fs::create_dir_all(&opts.data_dir)?;
	}

	// Database
	info!("Connecting to database");
	let pool = SqlitePoolOptions::new()
		.max_connections(5)
		.connect_with(
			// Connection options
			SqliteConnectOptions::new()
				.filename(opts.data_dir.join("sqlite.db"))
				.create_if_missing(true)
				.foreign_keys(true)
	).await?;
	sqlx::migrate!("./migrations")
		.run(&pool)
		.await?;

	// Bot Options
	let options = poise::FrameworkOptions {
		commands: vec![
			commands::global_commands(),
			commands::admin_commands(),
			commands::unserious_commands(),
			commands::role_commands()
		].into_iter().flatten().collect(),
		prefix_options: poise::PrefixFrameworkOptions {
			prefix: Some("~".into()),
			edit_tracker: Some(poise::EditTracker::for_timespan(std::time::Duration::from_secs(3600)).into()),
			..Default::default()
		},
		event_handler: |ctx, event, framework, data| Box::pin(events::event_handler(ctx, event, framework, data)),
		pre_command: |ctx: poise::Context<'_, Data, Report>| {
			Box::pin(async move {
				debug!("{} invoked {}", ctx.author().name, ctx.invocation_string());
			})
		},
		post_command: |ctx| {
			Box::pin(async move {
				debug!("Invocation {} from {} finished successfully.", ctx.invocation_string(), ctx.author().name);
			})
		},
		on_error: |error| Box::pin(async move {
			if let Err(e) = error::error_handler(error).await {
				error!("Failed to handle error: {e:#}");
			}
		}),
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
					#[cfg(not(target_os = "windows"))]
					{
						#[cfg(feature = "systemd")]
						sd_notify::notify(false, &[sd_notify::NotifyState::Ready]);

						use tokio::signal::unix::{signal, SignalKind};
						let mut sigint = signal(SignalKind::interrupt()).unwrap();
						let mut sigterm = signal(SignalKind::terminate()).unwrap();
						tokio::select! {
							_ = sigint.recv() => {},
							_ = sigterm.recv() => {}
						}

						#[cfg(feature = "systemd")]
						sd_notify::notify(true, &[sd_notify::NotifyState::Stopping]);
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
					info!("Shutting Down");
					db.close().await;
					sm.shutdown_all().await;
				});
				Ok(Data {
					db: pool
				})
			})
		})
		.build();

	let mut client = serenity::ClientBuilder::new(config.read_token()?, intents)
		.framework(framework)
		.await?;
	// 	 {
	// 	Ok(c) => c,
	// 	Err(e) => {
	// 		error!("Error with Discord client: {e:?}");
	// 		return Ok(())
	// 	}
	// };

	client.start_autosharded().await.map_err(Into::into)

	// if let Err(e) = client.start().await {
	// 	error!("Client error: {e:?}");
	// 	return Ok(())
	// }
	// Ok(())
}

fn install_tracing(
	options: &structs::Opts
) {
	use tracing::Level;
	use tracing_error::ErrorLayer;
	use tracing_subscriber::prelude::*;

	let verbosity = match options.verbose {
		0 => Level::INFO,
		1 => Level::DEBUG,
		_ => Level::TRACE
	};
	let fmt_layer = tracing_subscriber::fmt::layer()
		.with_writer(std::io::stdout.with_max_level(verbosity))
		.without_time()
		.with_target(false);

	tracing_subscriber::registry()
		.with(fmt_layer)
		.with(ErrorLayer::default())
		.init();
}
