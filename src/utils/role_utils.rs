use std::{
	collections::HashMap
};
use poise::serenity_prelude::{
	self as serenity,
	RoleId,
	Role,
};

use crate::{Context, Error};

pub async fn all_roles_select(ctx: Context<'_>, overlap: Option<HashMap<RoleId, Role>>) -> Result<Vec<serenity::CreateSelectMenuOption>, Error> {
	let hash = ctx.guild()
		.expect("Could not fetch guild from cache.")
		.roles;
	let mut list: Vec<serenity::CreateSelectMenuOption> = Vec::new();
	for (rid, r) in hash {
		if r.name != "@everyone" {
			let mut sel = false;
			if let Some(hm) = &overlap {
				for k in hm.keys() {
					if k == &rid {
						sel = true;
						break;
					}
				}
			}
			list.push(serenity::CreateSelectMenuOption::new(r.name, rid)
				.default_selection(sel)
				.to_owned())
		}
	};
	Ok(list)
}

// THIS NEEDS TO BE POKED AT SOME POINT
// pub fn roles_from_selected(selected: Vec<String>, roles: HashMap<RoleId, Role>) -> HashMap<RoleId, Role> {
// 	return roles.iter()
// 		.filter_map(|(k, v)| if selected.contains(&k.to_string()) {Some((*k, v.clone()))} else {None})
// 		.collect::<HashMap<RoleId, Role>>();
// }

pub async fn test() -> Result<(), Error> {
	Ok(())
}
