use poise::serenity_prelude as serenity;
use sqlx::{
	FromRow,
	Row,
	sqlite::SqliteRow
};
use crate::{
	Error,
	utils::concat
};

#[derive(Debug, Clone)]
pub struct GuildSettings {
	/// Whether the server is serious or not, default `true`.
	pub serious: bool,
	/// Whether the bot should respond to message events, default `false`.
	pub messages: bool,
	/// Whether the bot should manage roles, default `false`.
	pub roles: bool,
	/// Channel to manage roles from if applicable, default `None`.
	pub roles_channel: Option<serenity::ChannelId>
}
impl GuildSettings {
	pub async fn from_query(
		guild_id: serenity::GuildId,
		db: &sqlx::SqlitePool
	) -> Result<Self, Error> {
		sqlx::query_as("SELECT serious,messages,roles,roles_channel FROM guild_settings WHERE id = ?;")
			.bind(guild_id.get() as i64)
			.fetch_one(db)
			.await
			.map_err(|e| e.into())
	}

	pub async fn update(
		&self,
		guild_id: serenity::GuildId,
		db: &sqlx::SqlitePool
	) -> Result<(), Error> {
		sqlx::query("UPDATE servers SET serious = ?, messages = ?, roles = ?, roles_channel = ? WHERE id = ?;")
			.bind(self.serious)
			.bind(self.messages)
			.bind(self.roles)
			.bind(self.roles_channel.map(|id| id.get() as i64))
			.bind(guild_id.get() as i64)
			.execute(db)
			.await
			.map(|_| ())
			.map_err(|e| e.into())
	}

	pub fn as_array(&self) -> [(&str, bool); 3] {
		[
			("serious", self.serious),
			("messages", self.messages),
			("roles", self.roles)
		]
	}

	pub fn as_selectmenuoptions(&self) -> Vec<serenity::CreateSelectMenuOption> {
		let mut opts = Vec::new();
		for (name, value) in self.as_array() {
			opts.push(serenity::CreateSelectMenuOption::new(name[0..1].to_uppercase() + &name[1..], concat("enable_", name))
				.default_selection(value)
				.to_owned());
		}
		opts
	}
}
impl Default for GuildSettings {
	fn default() -> Self {
		Self {
			serious: true,
			messages: false,
			roles: false,
			roles_channel: None
		}
	}
}
impl FromRow<'_, SqliteRow> for GuildSettings {
	fn from_row(row: &SqliteRow) -> sqlx::Result<Self, sqlx::Error> {
		let mapped_roles_channel = match row.try_get::<Option<u64>, &str>("roles_channel") {
			Ok(raw) => raw.map(|inner| serenity::ChannelId::from(inner)),
			Err(e) => return Err(e)
		};

		Ok(
			Self {
				serious: row.try_get::<bool, &str>("serious")?,
				messages: row.try_get::<bool, &str>("messages")?,
				roles: row.try_get::<bool, &str>("roles")?,
				roles_channel: mapped_roles_channel
			}
		)
	}
}

#[derive(FromRow, Debug)]
pub struct Group {
	/// Name of the role group
	pub name: String,
	#[sqlx(try_from = "u64")]
	/// [`serenity::MessageId`] that the group is posted in.
	pub message: serenity::MessageId,
}
impl Group {
	pub async fn from_batch_query(
		guild_id: serenity::GuildId,
		db: &sqlx::SqlitePool
	) -> Result<Vec<Self>, Error> {
		sqlx::query_as("SELECT name,message FROM role_groups WHERE guild_id = ?;")
			.bind(guild_id.get() as i64)
			.fetch_all(db)
			.await
			.map_err(|e| e.into())
	}
}


#[derive(FromRow, Debug)]
pub struct Role {
	#[sqlx(try_from = "u64")]
	/// [`serenity::GuildId`] of the server.
	pub guild_id: serenity::GuildId,
	/// Name of the group that the role is under.
	pub group_name: String,
	#[sqlx(try_from = "u64")]
	/// [`serenity::RoleId`] of the role.
	pub role_id: serenity::RoleId
}
impl Role {
	pub async fn from_batch_query(
		guild_id: serenity::GuildId,
		db: &sqlx::SqlitePool
	) -> Result<Vec<Self>, Error> {
		sqlx::query_as("SELECT group_name,role_id FROM roles WHERE guild_id = ?;")
			.bind(guild_id.get() as i64)
			.fetch_all(db)
			.await
			.map_err(|e| e.into())
	}
}
