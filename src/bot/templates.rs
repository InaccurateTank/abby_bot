use poise::serenity_prelude as serenity;
use abby_utils::{db_structs, concat};
use crate::{EMBED_STD, EMBED_WAIT, EMBED_FAIL};

/// Makes a [`serenity::CreateEmbed`] and fills it out using the builder function.
pub fn state_embed(success: bool, text: &str) -> serenity::CreateEmbed {
	let mut e = serenity::CreateEmbed::default();
	builder_state_embed(&mut e, success, text);
	e
}

/// Sets the [`serenity::CreateEmbed`] with the state_embed values.
pub fn builder_state_embed(e: &mut serenity::CreateEmbed, success: bool, text: &str) {
	if success {
		e.title(":white_check_mark: Success :white_check_mark:");
		e.color(EMBED_STD);
	} else {
		e.title(":warning: Failure :warning:");
		e.color(EMBED_FAIL);
	}
	e.description(text);
}

/// Makes a [`serenity::CreateEmbed`] and fills it out using the builder function.
pub fn processing_embed() -> serenity::CreateEmbed {
	let mut e = serenity::CreateEmbed::default();
	builder_processing_embed(&mut e);
	e
}

/// Sets the [`serenity::CreateEmbed`] with the processing_embed values.
pub fn builder_processing_embed(e: &mut serenity::CreateEmbed) {
	e.color(EMBED_WAIT);
	e.description("Processing, please wait...");
}

pub fn rolelist_embed(ctx: &serenity::Context, group: &str, rolelist: Vec<db_structs::RoleEntry>) -> serenity::CreateEmbed {
	let mut rolelist: Vec<(String, u16)> = rolelist.into_iter().map(|r|
		(serenity::RoleId(r.id as u64).to_role_cached(ctx).unwrap().name, r.users)
	).collect();
	rolelist.sort_by(|a, b| {
		let a_l = a.0.to_lowercase();
		let b_l = b.0.to_lowercase();
		a_l.cmp(&b_l)
	});
	let mut e = serenity::CreateEmbed::default();
	e.title(group);
	e.color(EMBED_STD);
	let mut name_str = String::new();
	let mut user_str = String::new();
	for (name, id) in rolelist {
		name_str = abby_utils::concat(&name_str, &format!("{name}\n"));
		user_str = abby_utils::concat(&user_str, &format!("{id}\n"));
	}
	name_str = name_str.trim_end_matches('\n').to_string();
	user_str = user_str.trim_end_matches('\n').to_string();
	e.field("Name", name_str, true);
	e.field("Users", user_str, true);
	e
}

pub fn rolelist_components(group: &str) -> serenity::CreateComponents {
	let mut c = serenity::CreateComponents::default();
	c.create_action_row(|row| {
		row.create_button(|button| {
			button.custom_id(concat("roles.pick.", group));
			button.label("Pick");
			button.style(serenity::ButtonStyle::Primary)
		});
		row.create_button(|button| {
			button.custom_id(concat("roles.edit.", group));
			button.label("Edit");
			button.style(serenity::ButtonStyle::Secondary)
		})
		// row.create_button(|button| {
		// 	button.custom_id(concat("roles.remove.", group));
		// 	button.label("Remove");
		// 	button.style(serenity::ButtonStyle::Danger)
		// })
	});
	c
}
