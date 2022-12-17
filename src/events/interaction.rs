use std::str::FromStr;

use crate::Error;
use poise::serenity_prelude as serenity;

pub async fn roles_click(ctx: &serenity::Context, mci: &serenity::MessageComponentInteraction) -> Result<(), Error> {
	let data = &mci.data;

	match data.custom_id.as_str() {
		"rolelist.all" => {},
		_ => {}
	}


	if data.custom_id.starts_with("roleadd") {
		let (_, id) = data.custom_id.split_once('.')
			.unwrap();
		let mut remove = "Added";
		let rid = serenity::RoleId::from_str(id)
			.expect("Could not parse RoleId");

		for role in &mci.member.as_ref().unwrap().roles {
			if role == &rid {
				remove = "Removed";
			}
		}

		if remove == "Removed" {
			mci.member
				.clone()
				.expect("Could not find Interaction Member")
				.remove_role(&ctx, rid)
				.await?;
		} else {
			mci.member
				.clone()
				.expect("Could not find Interaction Member")
				.add_role(&ctx, rid)
				.await?
		}

		ctx.cache.guild(mci.guild_id.unwrap()).unwrap().roles;

		mci.create_interaction_response(&ctx, |f| {
			f.kind(serenity::InteractionResponseType::ChannelMessageWithSource)
				.interaction_response_data(|d| {
					d.ephemeral(true);
					d.content(format!("Role {} has been {}.", id, remove))
				})
		}).await?;
	}

	Ok(())
}
