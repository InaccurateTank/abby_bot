use poise::serenity_prelude as serenity;
use abby_utils::db_structs;
use crate::{EMBED_STD, EMBED_WAIT, EMBED_FAIL};

pub fn state_embed(success: bool, text: &str) -> serenity::CreateEmbed {
	let mut e = serenity::CreateEmbed::default();
	if success {
		e.title(":white_check_mark: Success :white_check_mark:");
		e.color(EMBED_STD);
	} else {
		e.title(":warning: Failure :warning:");
		e.color(EMBED_FAIL);
	}
	e.description(text);
	e
}

pub fn processing_embed() -> serenity::CreateEmbed {
	let mut e = serenity::CreateEmbed::default();
	e.color(EMBED_WAIT);
	e.description("Processing, please wait...");
	e
}

pub fn rolelist_embed(ctx: &serenity::Context, group: &str, rolelist: Vec<db_structs::RoleEntry>) -> serenity::CreateEmbed {
	let mut e = serenity::CreateEmbed::default();
	e.title(group);
	e.color(serenity::utils::Color::from_rgb(102, 51, 102));
	let mut name_str = String::new();
	let mut user_str = String::new();
	for r in rolelist {
		name_str = abby_utils::concat(&name_str, &format!("{}\n", serenity::RoleId(r.id as u64).to_role_cached(&ctx).unwrap().name));
		user_str = abby_utils::concat(&user_str, &format!("{}\n", r.users));
	}
	name_str = name_str.trim_end_matches('\n').to_string();
	user_str = user_str.trim_end_matches('\n').to_string();
	e.field("Name", name_str, true);
	e.field("Users", user_str, true);
	e
}
