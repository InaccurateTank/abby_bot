//! Hard error construction and handling module.
//!
//! Soft errors (Recoverable/Ignoreable) are handled in the commands and not here.

use color_eyre::{Report, Result};
use poise::serenity_prelude as serenity;
use poise::FrameworkError;
use thiserror::Error;
use tracing::{
	instrument,
	error,
	warn
};
use crate::templates::status;

#[derive(Error, Debug)]
pub enum ConfigError {
	#[error("Could not read the configuration file {path}")]
	MissingFile {
		path: Box<std::path::Path>,
		source: std::io::Error
	},
	#[error("Either token or token_file could not be read.")]
	Token
}

#[derive(Error, Debug)]
pub enum BotError {
	#[error("Somehow recieved wrong interaction ID during a filtered interaction wait.")]
	/// Bot recieved an interaction on a call that it shouldn't have.
	WrongInteraction,

	#[error("Interaction timed out, please try again.")]
	/// User waited too long to handle thier interaction prompt.
	InteractionTimedOut,

	/// Command or modal input is unparseable.
	#[error("Malformed input {0:?} is not parseable.")]
	MalformedInput(String),

	/// Interaction ID is malformed somehow.
	#[error("The ID of the interaction is malformed somehow.")]
	MalformedInteraction,

	/// Command or modal input is parseable but invalid.
	#[error("Input within argument {0:?} is not malformed but is invalid. Please double check the command and try again.")]
	InvalidInput(String),

	/// Error specific to the rewind command.
	#[error("Inputs are not in the same channel as each other, aborting.")]
	InputChannelMismatch,

	/// Channel is not available to post in.
	#[error("Channel lacks permissions for viewing, posting or both. Either change the permission overrides or choose a different channel.")]
	ChannelInaccessable,

	/// The roles channel is NULL in the database.
	#[error("Roles channel is not set and for safety will not be infered. In order to use this command please set the channel with \"/setup roles\".")]
	RolesChannelUnset,

	/// Failed to create the role list for whatever reason.
	#[error("Failed to create role list for group {0:?}.")]
	RoleListFailed(String),

	/// Somthing failed with the confirm modal.
	#[error("Failed to confirm via confirmation modal, aborting.")]
	UnconfirmedModal,

	/// Role already exists in a given guild.
	#[error("A role with an identical name already exists.")]
	RoleAlreadyExists,

	/// Generic error for missing permissions. Pretty much used entirely on longform interactions.
	#[error("Missing required permissions for component: {}", .0.get_permission_names().join(", ").to_ascii_uppercase())]
	InteractionMissingPermissions(serenity::Permissions),

	/// Commands can filter being in a guild, Interactions can't somehow.
	#[error("This interaction can only be created and used in a guild. If this is in a direct message somthing has gone wrong.")]
	InteractionNotInGuild,

	#[error("The user count thread has somehow been called before it was calculated.")]
	UserCountNotCalculated
}

#[derive(Error, Debug)]
#[error(transparent)]
pub struct UserError(#[from] pub Report);

#[instrument(skip_all)]
pub async fn error_handler<U>(
	error: poise::FrameworkError<'_, U, Report>,
) -> Result<()> {
	const USER_ERROR: &str = "If you think this was an error with the bot, please contact the developer.";
	const BOT_ERROR: &str = "This is undoubtedly an error with the bot, please contact the developer.";

	match error {
		FrameworkError::Setup { error, ..} =>
			error!("Failed to complete setup: {error:#}"),

		FrameworkError::EventHandler { error, event, .. } => {
			if error.is::<UserError>() {
				warn!("User made an error with event {:?}: {error:#}", event.snake_case_name());
			} else {
				error!("Failed to handle event {:?}: {error:#}", event.snake_case_name());
			}
		},

		FrameworkError::Command { error, ctx, .. } => {
			if error.is::<UserError>() {
				warn!("User made an error when invoking {:?}: {error:#}", ctx.invocation_string());
				ctx.send(
					error_reply().embed(
							status::warning(
								None,
								format!("An error has been made while invoking {:?}:\n```\n{error:#}\n```", ctx.invocation_string())
							).footer(serenity::CreateEmbedFooter::new(USER_ERROR))
						)
				).await?;
			} else {
				error!("An error occurred during the execution of {:?}: {error:#}", ctx.invocation_string());
				ctx.send(
					error_reply().embed(
						status::error(
							None,
							format!("An error occurred during the execution of {:?}:\n```\n{error:#}\n```", ctx.invocation_string())
						).footer(serenity::CreateEmbedFooter::new(BOT_ERROR))
					)
				).await?;
			}
		},

		FrameworkError::SubcommandRequired { ctx } => {
			warn!("User attempted to invoke {:?}, which requires a subcommand, without a subcommand.", ctx.invocation_string());
			ctx.send(
				error_reply().embed(
					status::error(
						None,
						format!(
							"Command {:?} must be used with the following subcommands:\n```\n{}\n```", ctx.invocation_string(), ctx.command()
							.subcommands
							.iter()
							.map(|s| {
								format!("- {}", s.qualified_name)
							})
							.collect::<Vec<String>>()
							.join("\n")
						)
					).footer(serenity::CreateEmbedFooter::new(BOT_ERROR))
				)
			).await?;
		},

		FrameworkError::CommandPanic { ctx, .. } => {
			error!("User invocation of {:?} has paniced.", ctx.invocation_string());
			ctx.send(
				error_reply().embed(
					status::error(
						Some("Panic"),
						"An extremely bad error has occured. Please contact the bot owner and tell them to check the logs."
					)
				)
			).await?;
		},

		FrameworkError::ArgumentParse { error, input, ctx, .. } => {
			let description = match input {
				Some(s) => format!("Failed to parse {s:?} from invocation of {:?}", ctx.invocation_string()),
				None => format!("Failed to parse argument from invocation of {:?}", ctx.invocation_string())
			};
			warn!("{description}: {error:?}");
			ctx.send(
				error_reply().embed(
					status::error(
						None,
						format!("{description}\n```{error:#}```")
					).footer(serenity::CreateEmbedFooter::new(BOT_ERROR))
				)
			).await?;
		},

		FrameworkError::CommandStructureMismatch { description, ctx, .. } => {
			error!("Structure mismatch between registered and programmed command for {}: {description}", ctx.command.qualified_name);
			ctx.send(
				error_reply().embed(
					status::error(
						None,
						format!("Structure mismatch between registered and programmed command for {}:\n```\n{description}\n```", ctx.command.qualified_name)
					).footer(serenity::CreateEmbedFooter::new(BOT_ERROR))
				)
			).await?;
		},

		FrameworkError::CooldownHit { remaining_cooldown, ctx, .. } => {
			warn!("User attempted {:?} while on cooldown.", ctx.invocation_string());
			ctx.send(
				error_reply().embed(
					status::warning(
						None,
						format!("You must wait **{} seconds** before attempting to invoke this command again.", remaining_cooldown.as_secs())
					).footer(serenity::CreateEmbedFooter::new(USER_ERROR))
				)
			).await?;
		},

		FrameworkError::MissingBotPermissions { missing_permissions, ctx, .. } => {
			warn!("Bot is missing permissions for {:?}: {missing_permissions}", ctx.invocation_string());
			ctx.send(
				error_reply().embed(
					status::warning(
						None,
						format!("Bot requires the following permissions in order to execute {:?}: {missing_permissions}", ctx.invocation_string())
					)
				)
			).await?;
		},

		FrameworkError::MissingUserPermissions { missing_permissions, ctx, .. } => {
			if let Some(missing_permissions) = missing_permissions {
				warn!("User tried executing {:?} without permissions: {missing_permissions}", ctx.invocation_string());
				ctx.send(
					error_reply().embed(
						status::warning(
							None,
							format!("You must have the following permissions to execute {:?}:\n```\n{missing_permissions:#}\n```", ctx.invocation_string())
						).footer(serenity::CreateEmbedFooter::new(USER_ERROR))
					)
				).await?;
			} else {
				warn!("User tried executing {:?} without permissions.", ctx.invocation_string());
				ctx.send(
					error_reply().embed(
						status::warning(
							None,
							format!("You do not have the permissions to execute {:?}.", ctx.invocation_string())
						).footer(serenity::CreateEmbedFooter::new(USER_ERROR))
					)
				).await?;
			}
		},

		FrameworkError::NotAnOwner { ctx, .. } => {
			warn!("Non-owner user attempted to invoke {:?}.", ctx.invocation_string());
			ctx.send(
				error_reply().embed(
					status::warning(
						None,
						format!("{:?} can only be invoked by the owner of the bot.", ctx.invocation_string())
					).footer(serenity::CreateEmbedFooter::new(USER_ERROR))
				)
			).await?;
		},

		FrameworkError::GuildOnly { ctx, .. } => {
			warn!("User attempted to invoke {:?} outside of a guild.", ctx.invocation_string());
			ctx.send(
				error_reply().embed(
					status::warning(
						None,
						format!("{:?} can only be invoked inside of a server.", ctx.invocation_string())
					).footer(serenity::CreateEmbedFooter::new(USER_ERROR))
				)
			).await?;
		},

		FrameworkError::DmOnly { ctx, .. } => {
			warn!("User attempted to invoke {:?} outside of a direct message.", ctx.invocation_string());
			ctx.send(
				error_reply().embed(
					status::warning(
						None,
						format!("{:?} can only be invoked inside of a direct message.", ctx.invocation_string())
					).footer(serenity::CreateEmbedFooter::new(USER_ERROR))
				)
			).await?;
		},

		FrameworkError::NsfwOnly { ctx, .. } => {
			warn!("User attempted to invoke {:?} outside of a NSFW channel.", ctx.invocation_string());
			ctx.send(
				error_reply().embed(
					status::warning(
						None,
						format!("{:?} can only be invoked inside of a NSFW channel.", ctx.invocation_string())
					).footer(serenity::CreateEmbedFooter::new(USER_ERROR))
				)
			).await?;
		},

		FrameworkError::CommandCheckFailed { error, ctx, .. } => {
			if let Some(error) = error {
				warn!("Check failed to run for invocation {:?}: {error:?}", ctx.invocation_string());
				ctx.send(
					error_reply().embed(
						status::error(
							None,
							format!("Check failed to run for invocation {:?}:\n```\n{error:?}\n```", ctx.invocation_string())
						).footer(serenity::CreateEmbedFooter::new(BOT_ERROR))
					)
				).await?;
			} else {
				warn!("Check failed to run invocation {:?}.", ctx.invocation_string());
			}
		},

		FrameworkError::DynamicPrefix { error, msg, .. } => {
			error!("Dynamic prefix failed for {msg:?}: {error:?}");
		},

		FrameworkError::UnknownCommand { prefix, msg_content, .. } => {
			warn!("Recognized prefix {prefix:?} but didn't recognize command {msg_content:?}.");
		},

		FrameworkError::UnknownInteraction { interaction, .. } => {
			warn!("Recieved interaction data for unknown command {:?}.", interaction.data.name);
		}
	}
	Ok(())
}

fn error_reply() -> poise::CreateReply {
	poise::CreateReply::default()
		.reply(true)
		.ephemeral(true)
}
