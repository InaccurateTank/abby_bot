use crate::{Context, Error};
use poise::serenity_prelude as serenity;

/// Creates a list of roles.
#[poise::command(
	guild_only,
	slash_command,
	required_permissions="MANAGE_ROLES",
	ephemeral
)]
pub async fn rolelist(
	ctx:Context<'_>
) -> Result<(), Error> {
	Ok(())
}
