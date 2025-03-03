use color_eyre::{Report, Result};
use poise::serenity_prelude as serenity;
use poise::FrameworkError;
use thiserror::Error;
use tracing::{
	instrument,
	error,
	warn
};

use crate::templates;

#[derive(Error, Debug)]
pub enum BotError {
	#[error("Somehow recieved wrong interaction ID during a filtered interaction wait.")]
	/// Bot recieved an interaction on a call that it shouldn't have.
	WrongInteraction,

	#[error("Interaction timed out, please try again.")]
	/// User waited too long to handle thier interaction prompt.
	InteractionTimedOut,

	#[error("Feature not enabled in guild.")]
	/// User waited too long to handle thier interaction prompt.
	FeatureNotEnabled,

	/// Command or modal input is unparseable.
	#[error("Malformed input \"{0:?}\" is not parseable.")]
	MalformedInput(String),

	/// Command or modal input is parseable but invalid.
	#[error("Input within argument \"{0}\" is not malformed but is invalid. Please double check the command and try again.")]
	InvalidInput(String),

	/// Error specific to the rewind command.
	#[error("Input \"from\" is not in the same channel as \"until\".")]
	RewindChannelMismatch,

	/// Channel is not available to post in.
	#[error("Channel \"{0}\" is inaccessable for posting in. Either change the permission overrides or choose a different channel.")]
	ChannelInaccessable(String),

	#[error("Roles channel is not set and for safety will not be infered. In order to use this command please set the channel with \"/setup roles\".")]
	RolesChannelUnset,

	#[error("Failed to create role list for group \"{0}\".")]
	RoleListFailed(String),

	#[error("Config file does not exist. Please fill out generated config file before running again.")]
	ConfigFileMissing
}

#[derive(Error, Debug)]
#[error(transparent)]
pub struct UserError(#[from] pub Report);

#[instrument(skip_all)]
pub async fn error_handler<U>(
	error: poise::FrameworkError<'_, U, Report>,
) -> Result<()> {
	use crate::templates;

	const USER_ERROR: &str = "If you think this was an error with the bot, please contact the developer.";
	const BOT_ERROR: &str = "This is undoubtedly an error with the bot, please contact the developer.";

	match error {
		FrameworkError::Setup { error, ..} =>
			error!("Failed to complete setup: {error:#}"),

		FrameworkError::EventHandler { error, event, .. } =>
			error!("Failed to handle event {:?}: {error:#}", event.snake_case_name()),

		FrameworkError::Command { error, ctx, .. } => {
			if error.is::<UserError>() {
				warn!("User made an error when invoking {:?}: {error:#}", ctx.invocation_string());
				ctx.send(
					create_reply(
						templates::Status::WARNING,
						format!("Made an error while invoking {:?}:\n```\n{error:#}\n```", ctx.invocation_string()),
						Some(USER_ERROR)
					)
				).await?;
			} else {
				error!("An error occurred during the execution of {:?}: {error:?}", ctx.invocation_string());
				ctx.send(
					create_reply(
						templates::Status::ERROR,
						format!("An error occurred during the execution of {:?}:\n```\n{error:#}\n```", ctx.invocation_string()),
						Some(BOT_ERROR)
					)
				).await?;
			}
		},

		FrameworkError::SubcommandRequired { ctx } => {
			warn!("User attempted to invoke {:?}, which requires a subcommand, without a subcommand.", ctx.invocation_string());
			ctx.send(poise::CreateReply::default()
					.embed(templates::status_embed(templates::Status::ERROR, format!("Command {:?} must be used with the following subcommands:\n\n{}",
						ctx.invocation_string(),
						ctx.command()
							.subcommands
							.iter()
							.map(|s| {
								format!("- {}", s.qualified_name)
							})
							.collect::<Vec<String>>()
							.join("\n"))
					)
						.footer(serenity::CreateEmbedFooter::new(USER_ERROR))
					)
					.reply(true)
					.ephemeral(true)
			).await?;
		},

		FrameworkError::CommandPanic { ctx, .. } => {
			error!("User invocation of {:?} has paniced.", ctx.invocation_string());
			ctx.send(poise::CreateReply::default()
				.embed(templates::status_embed(templates::Status::ERROR, "An extremely bad error has occured. Please contact the bot owner and tell them to check the logs."))
				.reply(true)
				.ephemeral(true)
			).await?;
		},

		FrameworkError::ArgumentParse { error, input, ctx, .. } => {
			let description = match input {
				Some(s) => format!("Failed to parse {s:?} from invocation of {:?}", ctx.invocation_string()),
				None => format!("Failed to parse argument from invocation of {:?}", ctx.invocation_string())
			};
			warn!("{description}: {error:?}");
			ctx.send(poise::CreateReply::default()
				.embed(templates::status_embed(templates::Status::ERROR, format!("{description}\n```{error:#}```"))
					.footer(serenity::CreateEmbedFooter::new(BOT_ERROR)))
				.reply(true)
				.ephemeral(true)
			).await?;
		},

		FrameworkError::CommandStructureMismatch { description, ctx, .. } => {},

		FrameworkError::CooldownHit { remaining_cooldown, ctx, .. } => {},

		FrameworkError::MissingBotPermissions { missing_permissions, ctx, .. } => {},

		FrameworkError::MissingUserPermissions { missing_permissions, ctx, .. } => {},

		FrameworkError::NotAnOwner { ctx, .. } => {
			warn!("Non-owner user attempted to invoke {:?}.", ctx.invocation_string());
			ctx.send(poise::CreateReply::default()
				.embed(templates::status_embed(templates::Status::ERROR, format!("{:?} can only be invoked by the owner of the bot.", ctx.invocation_string()))
					.footer(serenity::CreateEmbedFooter::new(USER_ERROR)))
				.reply(true)
				.ephemeral(true)
			).await?;
		},

		FrameworkError::GuildOnly { ctx, .. } => {
			warn!("User attempted to invoke {:?} outside of a guild.", ctx.invocation_string());
			ctx.send(poise::CreateReply::default()
				.embed(templates::status_embed(templates::Status::ERROR, format!("{:?} can only be invoked inside of a server.", ctx.invocation_string()))
					.footer(serenity::CreateEmbedFooter::new(USER_ERROR)))
				.reply(true)
				.ephemeral(true)
			).await?;
		},

		FrameworkError::DmOnly { ctx, .. } => {
			warn!("User attempted to invoke {:?} outside of a direct message.", ctx.invocation_string());
			ctx.send(poise::CreateReply::default()
				.embed(templates::status_embed(templates::Status::ERROR, format!("{:?} can only be invoked inside of a direct message.", ctx.invocation_string()))
					.footer(serenity::CreateEmbedFooter::new(USER_ERROR)))
				.reply(true)
				.ephemeral(true)
			).await?;
		},

		FrameworkError::NsfwOnly { ctx, .. } => {
			warn!("User attempted to invoke {:?} outside of a NSFW channel.", ctx.invocation_string());
			ctx.send(poise::CreateReply::default()
				.embed(templates::status_embed(templates::Status::ERROR, format!("{:?} can only be invoked inside of a NSFW channel.", ctx.invocation_string()))
					.footer(serenity::CreateEmbedFooter::new(USER_ERROR)))
				.reply(true)
				.ephemeral(true)
			).await?;
		},

		FrameworkError::CommandCheckFailed { error, ctx, .. } => {},

		FrameworkError::DynamicPrefix { error, ctx, msg, .. } => {},

		FrameworkError::UnknownCommand { ctx, msg, prefix, msg_content, framework, invocation_data, trigger, .. } => {},

		FrameworkError::UnknownInteraction { ctx, framework, interaction, .. } => {}
	}
	Ok(())
}

fn create_reply(
	status: templates::Status,
	description: impl Into<String>,
	footer: Option<impl Into<String>>
) -> poise::CreateReply {
	let mut embed = templates::status_embed(status, description);
	if let Some(f) = footer {
		embed = embed.footer(serenity::CreateEmbedFooter::new(f));
	}
	poise::CreateReply::default()
		.embed(embed)
		.reply(true)
		.ephemeral(true)
}
