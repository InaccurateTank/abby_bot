//! Embed templating module for the creation of standardized messages.
//!
//! Contains both the functions for creation along with the [Status] struct.

use color_eyre::Result;
use poise::serenity_prelude as serenity;
use serenity::{ CreateActionRow, CreateButton };
use crate::{
	structs,
	utils,
	EMBED_FAIL, EMBED_STD, EMBED_WAIT
};

#[derive(Debug)]
pub struct Status {
	symbol: &'static str,
	string: &'static str,
	color: serenity::Color
}

impl Status {
	pub const SUCCESS: Self = Self {
		symbol: ":white_check_mark:",
		string: "Success",
		color: EMBED_STD
	};
	pub const PROCESSING: Self = Self {
		symbol: ":gear:",
		string: "Processing",
		color: EMBED_WAIT
	};
	pub const WARNING: Self = Self {
		symbol: ":warning:",
		string: "Warning",
		color: EMBED_WAIT
	};
	pub const ERROR: Self = Self {
		symbol: ":exclamation:",
		string: "Error",
		color: EMBED_FAIL
	};
}

/// Templates [`serenity::CreateEmbed`] for bot status embeds.
pub fn status_embed(
	status: Status,
	description: impl Into<String>,
) -> serenity::CreateEmbed {
	serenity::CreateEmbed::new()
		.title(format!("{0} {1} {0}", status.symbol, status.string))
		.description(description)
		.color(status.color)
}

/// Makes a [`serenity::CreateEmbed`] and fills it out using the builder function.
pub fn state_embed(success: bool, text: &str) -> serenity::CreateEmbed {
	serenity::CreateEmbed::new()
		.description(text)
		.title(
			if success {":white_check_mark: Success :white_check_mark:"}
			else {":warning: Failure :warning:"}
		).color(
			if success {EMBED_STD}
			else {EMBED_FAIL}
		)
}

/// Makes a [`serenity::CreateEmbed`] and fills it out using the builder function.
pub fn processing_embed() -> serenity::CreateEmbed {
	serenity::CreateEmbed::new()
		.color(EMBED_WAIT)
		.description("Processing, please wait...")
}

pub fn rolelist_embed<'a>(
	// ctx: &serenity::Context,
	group: impl Into<String>,
	role_list: impl IntoIterator<Item = &'a structs::RoleVitals>
	// guild_id: serenity::GuildId
) -> Result<serenity::CreateEmbed> {
	// let mut rolelist: Vec<(String, u16)> = rolelist.into_iter().map(|r|
	// 	(guild_id.to_guild_cached(ctx)
	// 		.unwrap()
	// 		.roles
	// 		.get(&serenity::RoleId::new(r.id as u64))
	// 		.unwrap()
	// 		.name.clone(), r.users)
	// ).collect();
	// rolelist.sort_by(|a, b| {
	// 	let a_l = a.name.to_lowercase();
	// 	let b_l = b.name.to_lowercase();
	// 	a_l.cmp(&b_l)
	// });

	let mut name_str = String::new();
	let mut user_str = String::new();
	for r in role_list {
		name_str = utils::concat(&name_str, &format!("{}\n", r.name));
		user_str = utils::concat(&user_str, &format!("{}\n", r.users.get_calculated()?));
	}
	name_str = name_str.trim_end_matches('\n').to_string();
	user_str = user_str.trim_end_matches('\n').to_string();
	Ok(serenity::CreateEmbed::new()
		.title(group)
		.color(EMBED_STD)
		.fields([
			("Name", name_str, true),
			("Users", user_str, true)
		]))
}

pub fn rolelist_components(group: &str) -> serenity::CreateActionRow {
	CreateActionRow::Buttons(vec![
		CreateButton::new(utils::concat("roles.pick.", group))
			.label("Choose")
			.style(serenity::ButtonStyle::Primary),
		CreateButton::new(utils::concat("roles.edit.", group))
			.label("Modify")
			.style(serenity::ButtonStyle::Danger)
	])
}
