use std::{
	collections::HashMap
};
use poise::serenity_prelude::{
	self as serenity,
	RoleId,
	Role,
};

use crate::{Context, Error};

pub async fn all_roles_select(ctx: Context<'_>, overlap: Option<HashMap<RoleId, Role>>) -> Result<Vec<serenity::builder::CreateSelectMenuOption>, Error> {
	let hash = ctx.guild()
		.expect("Could not fetch guild from cache.")
		.roles;
	let mut list: Vec<serenity::builder::CreateSelectMenuOption> = Vec::new();
	for (rid, r) in hash {
		if r.name != "@everyone" {
			let mut sel = false;
			if let Some(hm) = &overlap {
				for (k, _) in hm {
					if k == &rid {
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

pub fn roles_from_selected(selected: Vec<String>, roles: HashMap<RoleId, Role>) -> HashMap<RoleId, Role> {
	return roles.iter()
		.filter_map(|(k, v)| if selected.contains(&k.to_string()) {Some((k.clone(), v.clone()))} else {None})
		.collect::<HashMap<RoleId, Role>>();
}
