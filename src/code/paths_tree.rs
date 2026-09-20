use crate::tree::{COL_PATH_SIZE_DEFAULT, COL_PATH_SIZE_MIN};

pub struct PathsTree {
	path_width: usize,
}

impl Default for PathsTree {
	fn default() -> Self {
		Self::new()
	}
}

impl PathsTree {
	pub fn new() -> Self {
		Self { path_width: COL_PATH_SIZE_DEFAULT } // <-- Domyślnie 35
	}

	pub fn path_width(mut self, width: usize) -> Self {
		self.path_width = width.max(COL_PATH_SIZE_MIN); // <-- Minimum 25
		self
	}

	pub fn format_path_chunks(&self, full_path: &str) -> Vec<String> {
		let width = self.path_width.max(COL_PATH_SIZE_MIN); // <-- Minimum 25
		if full_path.is_empty() {
			return vec![String::new()];
		}

		let chars: Vec<char> = full_path.chars().collect();
		chars.chunks(width).map(|chunk| chunk.iter().collect()).collect()
	}
}
