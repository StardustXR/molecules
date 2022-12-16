use stardust_xr_fusion::client::Client;

pub fn set_base_prefixes(client: &Client) {
	let mut dirs = Vec::new();
	#[cfg(feature = "dev")]
	dirs.push(directory_relative_path!("res").to_string());
	if let Ok(home) = std::env::var("HOME") {
		dirs.push(home + "/.local/share");
	}
	if let Ok(data_dir) = std::env::var("XDG_DATA_DIRS") {
		for dir in data_dir.split(':') {
			dirs.push(dir.to_string());
		}
	}
	dbg!(&dirs);
	client.set_base_prefixes(&dirs);
}
