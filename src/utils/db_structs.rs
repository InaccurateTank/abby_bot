use poise::serenity_prelude as serenity;
use sqlx::FromRow;
use crate::concat;

// Server Features
#[derive(FromRow, Debug)]
pub struct Server {
	/// ID of the server bitcast as an [i64].
	pub srvid: i64,
	/// Whether the server is serious or not, defaults to `true`.
	pub serious: bool,
	/// Whether the bot should respond to message events, defaults to `false`.
	pub messages: bool,
	/// Whether the bot should manage roles, defaults to `false`.
	pub roles: bool
}
impl Default for Server {
	fn default() -> Self {
		Server {
			srvid: 0,
			serious: true,
			messages: false,
			roles: false
		}
	}
}
impl Server {
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

// Primary Roles Managment
#[derive(Default, FromRow, Debug)]
pub struct ServerRoles {
	/// ID of the server bitcast as an [i64].
	pub srvid: i64,
	/// Table for all the servers managed roles as a [`String`].
	pub tab: String,
	/// ID of the server bitcast as an [i64].
	pub channel: Option<i64>
}

// #[derive(Debug)]
// pub struct RoleList(pub Vec<RoleEntry>);
// impl RoleList {
// 	pub fn to_roles(&self, ctx: &serenity::Context) -> Vec<serenity::Role> {
// 		self.0.iter()
// 			.map(|f| serenity::RoleId(f.id as u64).to_role_cached(ctx).unwrap())
// 			.collect()
// 	}
// 	pub async fn from_roleids(&self, ctx: &serenity::Context, db: &sqlx::Pool<sqlx::Sqlite>, roles: Vec<serenity::RoleId>) -> Self {
// 		let res = Vec::new();
// 		for rid in roles {
// 			res.push(
// 				sqlx::query_as::<_, RoleEntry>(&format!("SELECT * FROM roles_{srv_id} WHERE grp = ?;"))
// 					.bind(value)
// 					.fetch_all(db)
// 					.await
// 					.unwrap();
// 			)
// 		}
// 		RoleList(res)
// 	}
// }

// Individual Role List Struct
#[derive(Default, FromRow, Debug)]
pub struct RoleEntry {
	/// The ID of the role bitcast as an [i64].
	pub id: i64,
	/// [`String`] of the group that the role belongs to.
	pub grp: String,
	/// Roughly the amount of users with the role.
	pub users: u16
}
