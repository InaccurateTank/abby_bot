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

			// ctx.cache.guild(mci.guild_id.unwrap()).unwrap().roles;
			mci.create_interaction_response(&ctx, |f| {
				f.kind(serenity::InteractionResponseType::ChannelMessageWithSource).interaction_response_data(|d| {
						d.ephemeral(true);
						d.content(format!("Role {back} has been {remove}."))
					})
				}).await?;
		}
		"edit" => {
			// let mut list: Vec<String> = Vec::new();
			for row in &mci.message.components {
				for arc in &row.components {
					if let serenity::ActionRowComponent::Button(button) = arc {
						let id = &**button
							.custom_id
							.as_ref()
							.unwrap();
						let (f, b) = id
							.split_once('.')
							.unwrap();
						if f == "roleadd" {
							println!("{b}");
							// list.push(b.to_string());
						}
					}
				}
			}
		},
		_ => {
			inter_err(":warning: Unknown Interaction ID :warning:", &format!("Unknown Interaction ID {}", &data.custom_id), ctx, mci).await?;
			return Ok(());
		}
	}
	Ok(())
}
