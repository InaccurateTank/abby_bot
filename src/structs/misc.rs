use color_eyre::{eyre::eyre, Result};
use poise::serenity_prelude as serenity;
use tracing::warn;
use crate::error::BotError;

pub enum UserCount {
	Working(tokio::task::JoinHandle<u8>),
	Calculated(u8)
}
impl UserCount {
	pub fn get(&self) -> Result<&u8> {
		if let Self::Calculated(v) = self {
			Ok(v)
		} else {
			Err(BotError::UserCountNotCalculated.into())
		}
	}

	pub fn get_mut(&mut self) -> Result<&mut u8> {
		if let Self::Calculated(v) = self {
			Ok(v)
		} else {
			Err(BotError::UserCountNotCalculated.into())
		}
	}

	pub async fn solve(&mut self) -> Result<()> {
		if let Self::Working(result) = self {
			*self = Self::Calculated(result.await?);
		} else {
			warn!("Attempt to solve already solved user count.");
		}
		Ok(())
	}

	pub fn decrement(&mut self) -> Result<()> {
		let v = self.get_mut()?;
		*v = v.saturating_sub(1);
		Ok(())
	}

	pub fn increment(&mut self) -> Result<()> {
		let v = self.get_mut()?;
		*v = v.saturating_add(1);
		Ok(())
	}
}

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
		let spawn = tokio::task::spawn_blocking(move || {
			closure_members.into_iter()
				.filter(|(_, m)| m.roles.contains(&closure_id)).count() as u8
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
