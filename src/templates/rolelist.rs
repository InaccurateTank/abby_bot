//! Templating submodule for all role list message items.

use color_eyre::Result;
use poise::serenity_prelude as serenity;
use crate::{
	colors,
	structs,
	utils
};

pub fn embed<'a>(
	group: impl Into<String>,
	role_list: impl IntoIterator<Item = &'a structs::RoleVitals>
) -> Result<serenity::CreateEmbed> {
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
		.color(colors::INFO)
		.fields([
			("Name", name_str, true),
			("Users", user_str, true)
		]))
}

pub fn components(group: &str) -> serenity::CreateActionRow {
	serenity::CreateActionRow::Buttons(vec![
		serenity::CreateButton::new(utils::concat("roles.pick.", group))
			.label("Choose")
			.style(serenity::ButtonStyle::Primary),
			serenity::CreateButton::new(utils::concat("roles.edit.", group))
			.label("Modify")
			.style(serenity::ButtonStyle::Danger)
	])
}
