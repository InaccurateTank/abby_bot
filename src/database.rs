use color_eyre::Result;
use poise::serenity_prelude as serenity;
use sqlx::{
	FromRow,
	Row,
	sqlite::SqliteRow
};
use crate::concat;

#[derive(Default)]
pub struct GuildSettings {
	/// Whether the server will allow unserious and/or meme content or not, default `true`.
	pub unserious: bool,
	/// Whether the bot should allow generic administration work commands.
	pub admin: bool,
	/// Whether the bot should respond to message events, default `false`.
	pub messages: bool,
	/// Whether the bot should manage roles, default `false`.
	pub roles: bool,
	/// Channel to manage roles from if applicable, default `None`.
	pub roles_channel: Option<serenity::ChannelId>
}
impl GuildSettings {
	/// Creates an instance of the object from a database query.
	pub async fn from_query(
		guild_id: serenity::GuildId,
		db: &sqlx::SqlitePool
	) -> Result<Self> {
		sqlx::query_as("SELECT admin,unserious,messages,roles,roles_channel FROM guild_settings WHERE guild_id = ?;")
			.bind(guild_id.get() as i64)
			.fetch_one(db)
			.await
			.map_err(Into::into)
	}

	/// While the standard default function is useful for servers, some minor changes are needed for handling private messages.
	pub fn private_default() -> Self {
		Self {
			admin: false,
			unserious: false,
			messages: true,
			roles: false,
			roles_channel: None
		}
	}

	/// Updates the database entry of a given [GuildId]['serenity::GuildId'] with the values contained in the struct.
	pub async fn update(
		&self,
		guild_id: serenity::GuildId,
		db: &sqlx::SqlitePool
	) -> Result<sqlx::sqlite::SqliteQueryResult> {
		sqlx::query("UPDATE guild_settings SET admin = ?, unserious = ?, messages = ?, roles = ?, roles_channel = ? WHERE guild_id = ?;")
			.bind(self.admin)
			.bind(self.unserious)
			.bind(self.messages)
			.bind(self.roles)
			.bind(self.roles_channel.map(|id| id.get() as i64))
			.bind(guild_id.get() as i64)
			.execute(db)
			.await
			// .map(|_| ())
			.map_err(Into::into)
	}

	/// All true/false options as an array.
	pub fn as_array(&self) -> [(&str, bool); 4] {
		[
			("admin", self.admin),
			("unserious", self.unserious),
			("messages", self.messages),
			("roles", self.roles)
		]
	}

	/// All true/false options as [CreateSelectMenuOption]['serenity::CreateSelectMenuOption'] components.
	pub fn as_selectmenuoptions(&self) -> Vec<serenity::CreateSelectMenuOption> {
		let mut opts = Vec::new();
		for (name, value) in self.as_array() {
			opts.push(serenity::CreateSelectMenuOption::new(name[0..1].to_uppercase() + &name[1..], concat!("enable_", name))
				.default_selection(value)
				.to_owned());
		}
		opts
	}
}
impl FromRow<'_, SqliteRow> for GuildSettings {
	fn from_row(row: &SqliteRow) -> sqlx::Result<Self, sqlx::Error> {
		let mapped_roles_channel = match row.try_get::<Option<u64>, &str>("roles_channel") {
			Ok(raw) => raw.map(serenity::ChannelId::from),
			Err(e) => return Err(e)
		};

		Ok(
			Self {
				admin: row.try_get::<bool, &str>("admin")?,
				unserious: row.try_get::<bool, &str>("unserious")?,
				messages: row.try_get::<bool, &str>("messages")?,
				roles: row.try_get::<bool, &str>("roles")?,
				roles_channel: mapped_roles_channel
			}
		)
	}
}

pub async fn roles_channel_query(
	guild_id: serenity::GuildId,
	db: &sqlx::SqlitePool
) -> Result<Option<serenity::ChannelId>> {
	sqlx::query_scalar::<_, Option<u64>>("SELECT roles_channel FROM guild_settings WHERE guild_id = ?;")
		.bind(guild_id.get() as i64)
		.fetch_one(db)
		.await
		.map_or_else(
			|e| Err(e.into()),
			|v| Ok(v.map(serenity::ChannelId::from))
		)
}

pub enum CheckGuildSetting {
	Admin,
	Role,
	Unserious
}
impl CheckGuildSetting {
	pub async fn query(
		&self,
		guild_id: &serenity::GuildId,
		db: &sqlx::SqlitePool
	) -> Result<bool> {
		let text = match self {
			Self::Admin => "SELECT admin FROM guild_settings WHERE guild_id = ?;",
			Self::Role => "SELECT roles FROM guild_settings WHERE guild_id = ?;",
			Self::Unserious => "SELECT unserious FROM guild_settings WHERE guild_id = ?;",
		};
		sqlx::query_scalar::<_, bool>(text)
			.bind(guild_id.get() as i64)
			.fetch_one(db)
			.await
			.map_err(Into::into)
	}
}

pub async fn select_guilds(
	db: &sqlx::SqlitePool
) -> Result<std::collections::HashSet<u64>> {
	let result = sqlx::query_scalar::<_, u64>("SELECT guild_id FROM guild_settings;")
		.fetch_all(db)
		.await;
	match result {
		Ok(r) => Ok(r.into_iter().collect()),
		Err(e) => Err(e.into()),
	}
}

pub async fn insert_guild(
	guild_id: serenity::GuildId,
	db: &sqlx::SqlitePool
) -> Result<sqlx::sqlite::SqliteQueryResult> {
	sqlx::query("INSERT INTO guild_settings (guild_id) VALUES (?);")
		.bind(guild_id.get() as i64)
		.execute(db)
		.await
		.map_err(Into::into)
}

pub async fn delete_guild(
	guild_id: serenity::GuildId,
	db: &sqlx::SqlitePool
) -> Result<sqlx::sqlite::SqliteQueryResult> {
	sqlx::query("DELETE FROM guild_settings WHERE guild_id = ?;")
		.bind(guild_id.get() as i64)
		.execute(db)
		.await
		.map_err(Into::into)
}

// Role Groups Table

#[derive(FromRow)]
pub struct Group {
	/// Name of the role group
	pub group_name: String,
	#[sqlx(try_from = "u64")]
	/// [`serenity::MessageId`] that the group is posted in.
	pub message_id: serenity::MessageId,
}

pub async fn group_message_query(
	guild_id: serenity::GuildId,
	group_name: &str,
	db: &sqlx::SqlitePool
) -> Result<serenity::MessageId> {
	sqlx::query_scalar::<_, u64>("SELECT message_id FROM role_groups WHERE guild_id = ? AND group_name = ?;")
		.bind(guild_id.get() as i64)
		.bind(group_name)
		.fetch_one(db)
		.await
		.map_or_else(
			|e| Err(e.into()),
			|v| Ok(serenity::MessageId::from(v))
		)
}

pub async fn groups_from_query(
	guild_id: serenity::GuildId,
	db: &sqlx::SqlitePool
) -> Result<Vec<Group>> {
	sqlx::query_as("SELECT group_name,message_id FROM role_groups WHERE guild_id = ?;")
		.bind(guild_id.get() as i64)
		.fetch_all(db)
		.await
		.map_err(Into::into)
}

// Roles Table

pub async fn roles_from_query(
	guild_id: serenity::GuildId,
	group_name: &str,
	db: &sqlx::SqlitePool
) -> Result<Vec<serenity::RoleId>> {
	let res = sqlx::query_scalar::<_, u64>("SELECT role_id FROM roles WHERE guild_id = ? AND group_name = ?;")
		.bind(guild_id.get() as i64)
		.bind(group_name)
		.fetch_all(db)
		.await;

	match res {
		Ok(v) => Ok(v.into_iter()
			.map(serenity::RoleId::from)
			.collect()),
		Err(e) => Err(e.into())
	}
}
