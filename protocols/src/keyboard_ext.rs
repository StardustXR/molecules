use std::{error::Error, fmt::Display};

use crate::keyboard::KeymapExchangeError;

impl Display for KeymapExchangeError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			KeymapExchangeError::InvalidKeymap => f.write_str("Invalid Keymap"),
		}
	}
}
impl Error for KeymapExchangeError {}
