use color_eyre::Result;
use poise::serenity_prelude as serenity;
use serenity::{ CreateActionRow, CreateButton };
use crate::{
	structs::misc::RoleVitals,
	utils,
	EMBED_FAIL, EMBED_STD, EMBED_WAIT
};

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
	role_list: impl IntoIterator<Item = &'a RoleVitals>
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
