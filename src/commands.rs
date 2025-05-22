use color_eyre::{Result, Report};
use poise::serenity_prelude as serenity;
use crate::{Context, Data};

mod bottomify;
use bottomify::*;

mod setup;
use setup::*;

mod rolelist;
use rolelist::*;

mod snap;
use snap::*;

mod rewind;
use rewind::*;

/// Vector of globally registered commands.
pub fn global_commands() -> Vec<poise::Command<Data, Report>> {
	vec![
		help(),
		about(),
		setup()
	]
}

/// Vector of administration commands.
pub fn admin_commands() -> Vec<poise::Command<Data, Report>> {
	vec![
		snap(),
		rewind()
	]
}


/// Vector of role management commands.
pub fn role_commands() -> Vec<poise::Command<Data, Report>> {
	vec![
		rolelist()
	]
}

/// Vector of meme and otherwise unserious commands.
pub fn unserious_commands() -> Vec<poise::Command<Data, Report>> {
	vec![
		bottomify()
	]
}

const AUTHORS: &str = env!("CARGO_PKG_AUTHORS");
const VERSION: &str = env!("CARGO_PKG_VERSION");
const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");
const REPO: &str = env!("CARGO_PKG_REPOSITORY");

/// Displays this command help prompt.
#[poise::command(
	prefix_command, slash_command,
	category="General",
	track_edits
)]
async fn help(
	ctx: Context<'_>,
	#[description = "Specific command to show help about"]
	#[autocomplete = "poise::builtins::autocomplete_command"]
	command: Option<String>
) -> Result<()> {
	let config = poise::builtins::HelpConfiguration {
		extra_text_at_bottom: "\
If this is used as a prefix command, the invoking message can be edited to change my response.",
		..Default::default()
	};
	poise::builtins::help(ctx, command.as_deref(), config).await?;
	Ok(())
}

/// About the bot.
#[poise::command(
	prefix_command, slash_command,
	category="General",
	ephemeral
)]
async fn about(
	ctx:Context<'_>
) -> Result<()> {
	ctx.send(poise::CreateReply::default()
		.embed(serenity::CreateEmbed::new()
			.color(serenity::Color::new(663366))
			.description(DESCRIPTION)
			.field("Creator", AUTHORS.replace(':', "\n"), true)
			.field("Version", VERSION, true)
			.field("Repository", REPO, false)
			.footer(serenity::CreateEmbedFooter::new("Made with incompetence")))
	).await?;
	Ok(())
}
