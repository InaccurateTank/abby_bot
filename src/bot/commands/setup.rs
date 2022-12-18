use abby_utils::{Context, Error};

/// A command with two subcommands: `child1` and `child2`
///
/// Running this function directly, without any subcommand, is only supported in prefix commands.
/// Discord doesn't permit invoking the root command of a slash command if it has subcommands.
#[poise::command(
	guild_only,
	slash_command,
	required_permissions="ADMINISTRATOR",
	category="Administration",
	ephemeral,
	subcommands("commands", "roles")
)]
pub async fn setup(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("You shouldn't be here?").await?;
    Ok(())
}

/// A subcommand of `parent`
#[poise::command(prefix_command, slash_command)]
pub async fn commands(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("You invoked the first child command!").await?;
    Ok(())
}

/// Another subcommand of `parent`
#[poise::command(prefix_command, slash_command)]
pub async fn roles(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("You invoked the second child command!").await?;
    Ok(())
}

