use poise::serenity_prelude as serenity;
use sqlx::FromRow;
use crate::concat;

// Server Features
#[derive(FromRow, Debug)]
pub struct Server {
	pub srvid: i64,
	pub serious: bool,
	pub messages: bool,
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
pub struct ServerRoles {
	pub srvid: i64,
	pub tab: String,
	pub channel: i64
}
impl ServerRoles {

}
