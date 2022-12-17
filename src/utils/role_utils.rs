use poise::serenity_prelude as serenity;

use crate::{Context, Error};

pub async fn all_roles_select(ctx: Context<'_>, overlap: Option<Vec<String>>) -> Result<Vec<serenity::builder::CreateSelectMenuOption>, Error>{
	let hash = ctx.guild()
		.expect("Could not fetch guild from cache.")
		.roles;
	let mut list: Vec<serenity::builder::CreateSelectMenuOption> = Vec::new();
	for (rid, r) in hash {
		if r.name != "@everyone" {
			let mut sel = false;
			if let Some(v) = &overlap {
				for second in v {
					if second == &rid.to_string() {
						sel = true;
						break;
					}
				}
			}
			list.push(serenity::builder::CreateSelectMenuOption::new(r.name, rid)
				.default_selection(sel)
				.to_owned())
		}
	};
	Ok(list)
}
