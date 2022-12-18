use std::str::FromStr;
use poise::serenity_prelude as serenity;
use abby_utils::{Error, inter_err};

pub async fn roles_click(ctx: &serenity::Context, mci: &serenity::MessageComponentInteraction) -> Result<(), Error> {
	let data = &mci.data;
	let (front, back) = data.custom_id.split_once('.')
		.expect("Not a splittable ID");

	match front {
		"roleadd" => {
			let mut remove = "Added";
			let rid = serenity::RoleId::from_str(back)
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
				f.kind(serenity::InteractionResponseType::ChannelMessageWithSource).interaction_response_data(|d| {
						d.ephemeral(true);
						d.content(format!("Role {} has been {}.", back, remove))
					})
				}).await?;
		}
		_ => {
			inter_err(":warning: Unknown Interaction ID :warning:", format!("Unknown Interaction ID {}", &data.custom_id).as_str(), ctx, mci).await?;
			return Ok(());
		}
	}
	Ok(())
}
