use crate::{Context, Error};
use poise::serenity_prelude as serenity;

const AUTHORS: &'static str = env!("CARGO_PKG_AUTHORS");
const VERSION: &'static str = env!("CARGO_PKG_VERSION");
const DESCRIPTION: &'static str = env!("CARGO_PKG_DESCRIPTION");
const REPO: &'static str = env!("CARGO_PKG_REPOSITORY");

/// Displays this command help prompt.
#[poise::command(
	prefix_command, slash_command,
	category="General",
	track_edits
)]
pub async fn help(
	ctx: Context<'_>,
	#[description = "Specific command to show help about"]
	#[autocomplete = "poise::builtins::autocomplete_command"]
	command: Option<String>,
) -> Result<(), Error> {
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
pub async fn about(
	ctx:Context<'_>
) -> Result<(), Error> {
	ctx.send(|b| {
		b.content("")
		.embed(|e|{
			e.title("Hello I'm Abby!")
				.color(serenity::utils::Color::new(663366))
				.description(DESCRIPTION)
				.field("Creator", AUTHORS.replace(":", "\n"), true)
				.field("Version", VERSION, true)
				.field("Repository", REPO, false)
				.footer(|f| {
					f.text("Made with incompetence")
				})
		})
	}).await?;

	ctx.send(|b| {
		b.content("")
		.embed(|e|{
			e.color(serenity::utils::Color::LIGHTER_GREY);
			e.field("Pronouns", "She/Her\nHe/Him\nThey/Them\nIt/Its", true);
			e.field("Users", "2\n0\n3\n0", true)
		});
		b.components(|c| {
			c.create_action_row(|row| {
				row.create_button(|button| {
					button.custom_id("roleadd.roles");
					button.label("Select Roles");
					button.style(serenity::ButtonStyle::Primary)
				});
				row.create_button(|button| {
					button.custom_id("edit.roles");
					button.label("Edit Roles");
					button.style(serenity::ButtonStyle::Secondary)
				})
			})
		})
	}).await?;
	Ok(())
}

/// Registers slash commands either within this server or globally. Only usable by the bot owner.
#[poise::command(
	prefix_command,
	category="General", hide_in_help,
	owners_only
)]
pub async fn register(ctx: Context<'_>) -> Result<(), Error> {
	poise::builtins::register_application_commands_buttons(ctx).await?;
	Ok(())
}
