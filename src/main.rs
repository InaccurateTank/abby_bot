use poise::serenity_prelude as serenity;

mod commands;
mod events;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

pub struct Data {} // User data, which is stored and accessible in all command invocations

#[tokio::main]
	async fn main() {
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
					poise::Event::Ready { data_about_bot: ready } => {
						println!("{} is connected!", ready.user.name);
						ctx.set_activity(serenity::Activity::watching("Everything")).await;
					},
					poise::Event::Message { new_message: msg } => {
						if !msg.is_own(&ctx.cache) {
							let m = msg.content.as_str();
							events::borger(ctx, msg, &m).await?;
							events::v(ctx, msg, &m).await?;
						}
					},
					_ => ()
				}
				Ok(())
			})
		},
		..Default::default()
	};
	let token = "OTc2MzQwMTM3NjMxOTYxMDg4.GpM-Gs.0kOy37LTuH3FtBOomK97BWB8B9U6PxnomDfqlA";
	let intents = serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::GUILD_MESSAGES | serenity::GatewayIntents::MESSAGE_CONTENT;

	let framework = poise::Framework::builder()
		.options(options)
		.token(token)
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
				Ok(Data {})
			})
		});

	framework.run()
		.await
		.unwrap();
}
