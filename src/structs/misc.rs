use color_eyre::{eyre::eyre, Result};
use poise::serenity_prelude as serenity;

#[derive(Debug)]
pub enum UserCount {
	Working(tokio::task::JoinHandle<u8>),
	Calculated(u8)
}
impl UserCount {
	pub fn get_calculated(&self) -> Result<&u8> {
		if let Self::Calculated(v) = &self {
			Ok(v)
		} else {
			// TODO: Better Errors
			Err(eyre!("Not Calculated"))
		}
	}

	pub async fn solve(&mut self) -> Result<()> {
		if let Self::Working(result) = self {
			*self = Self::Calculated(result.await?)
		} else {
			// TODO: Better Errors
			return Err(eyre!("Already Calculated"))
		}
		Ok(())
	}

	pub fn decrement(&mut self) -> Result<()> {
		if let Self::Calculated(v) = self {
			*v = v.saturating_sub(1);
		} else {
			// TODO: Better Errors
			return Err(eyre!("Not Calculated"))
		}
		Ok(())
	}

	pub fn increment(&mut self) -> Result<()> {
		if let Self::Calculated(v) = self {
			*v = v.saturating_add(1);
		} else {
			// TODO: Better Errors
			return Err(eyre!("Not Calculated"))
		}
		Ok(())
	}
}

#[derive(Debug)]
pub struct RoleVitals {
	pub id: serenity::RoleId,
	pub name: String,
	pub users: UserCount
}
impl RoleVitals {
	pub fn new(
		id: serenity::RoleId,
		guild: &serenity::Guild
	) -> Result<Self> {
		let closure_id = id.to_owned();
		let closure_members = guild.members.to_owned();
		let spawn = tokio::spawn(async move {
			closure_members.into_iter()
				.filter(|(_, m)| {
					m.roles.contains(&closure_id)
				}).count() as u8
		});

		let name = guild.roles
			.get(&id)
			.ok_or(eyre!("No RoleId"))?
			.name
			.to_owned();

		Ok(RoleVitals {
			id,
			name,
			users: UserCount::Working(spawn)
		})
	}
}
