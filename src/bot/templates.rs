use poise::serenity_prelude as serenity;
use serenity::{ CreateActionRow, CreateButton };
use crate::utils::concat;
use crate::structs::db;
// use abby_utils::{db_structs, concat};
use crate::{EMBED_STD, EMBED_WAIT, EMBED_FAIL};

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

/// Sets the [`serenity::CreateEmbed`] with the state_embed values.
// pub fn builder_state_embed(e: &mut serenity::CreateEmbed, success: bool, text: &str) {
// 	if success {
// 		e.title(":white_check_mark: Success :white_check_mark:");
// 		e.color(EMBED_STD);
// 	} else {
// 		e.title(":warning: Failure :warning:");
// 		e.color(EMBED_FAIL);
// 	}
// 	e.description(text);
// }

/// Makes a [`serenity::CreateEmbed`] and fills it out using the builder function.
pub fn processing_embed() -> serenity::CreateEmbed {
	serenity::CreateEmbed::new()
		.color(EMBED_WAIT)
		.description("Processing, please wait...")
}

// Sets the [`serenity::CreateEmbed`] with the processing_embed values.
// pub fn builder_processing_embed(e: &mut serenity::CreateEmbed) {
// 	e.color(EMBED_WAIT);
// 	e.description("Processing, please wait...");
// }

pub fn rolelist_embed(ctx: &serenity::Context, group: &str, rolelist: Vec<db::RoleEntry>) -> serenity::CreateEmbed {
	let mut rolelist: Vec<(String, u16)> = rolelist.into_iter().map(|r|
		(serenity::RoleId::new(r.id as u64).to_role_cached(ctx).unwrap().name, r.users)
	).collect();
	rolelist.sort_by(|a, b| {
		let a_l = a.0.to_lowercase();
		let b_l = b.0.to_lowercase();
		a_l.cmp(&b_l)
	});
	let mut name_str = String::new();
	let mut user_str = String::new();
	for (name, id) in rolelist {
		name_str = concat(&name_str, &format!("{name}\n"));
		user_str = concat(&user_str, &format!("{id}\n"));
	}
	name_str = name_str.trim_end_matches('\n').to_string();
	user_str = user_str.trim_end_matches('\n').to_string();
	serenity::CreateEmbed::new()
		.title(group)
		.color(EMBED_STD)
		.fields([
			("Name", name_str, true),
			("Users", user_str, true)
		])
}

pub fn rolelist_components(group: &str) -> serenity::CreateActionRow {
	CreateActionRow::Buttons(vec![
		CreateButton::new(concat("roles.pick.", group))
			.label("Choose")
			.style(serenity::ButtonStyle::Primary),
		CreateButton::new(concat("roles.edit.", group))
			.label("Modify")
			.style(serenity::ButtonStyle::Danger)
	])
}
