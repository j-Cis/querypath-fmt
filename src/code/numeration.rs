#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Numeration {
	pub enabled: bool,
	pub numerate_dirs: bool,
	pub numerate_binaries: bool,
	pub start_from: usize,
}

impl Default for Numeration {
	fn default() -> Self {
		Self { enabled: true, numerate_dirs: true, numerate_binaries: true, start_from: 1 }
	}
}

impl Numeration {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn enabled(mut self, enabled: bool) -> Self {
		self.enabled = enabled;
		self
	}

	pub fn numerate_dirs(mut self, numerate: bool) -> Self {
		self.numerate_dirs = numerate;
		self
	}

	pub fn numerate_binaries(mut self, numerate: bool) -> Self {
		self.numerate_binaries = numerate;
		self
	}

	pub fn start_from(mut self, start: usize) -> Self {
		self.start_from = start;
		self
	}

	pub fn should_numerate(&self, is_dir: bool, is_binary: bool) -> bool {
		if self.enabled == false {
			return false;
		}
		if is_dir && self.numerate_dirs == false {
			return false;
		}
		if is_dir == false && is_binary && self.numerate_binaries == false {
			return false;
		}
		true
	}

	pub fn column_width(&self, max_num: usize) -> usize {
		if self.enabled == false || max_num == 0 { 0 } else { max_num.to_string().len() }
	}

	pub fn format_cell(&self, current_num: usize, max_num: usize) -> String {
		if self.enabled == false || max_num == 0 {
			return String::new();
		}
		let width = max_num.to_string().len();
		format!("{:>width$}", current_num, width = width)
	}

	pub fn empty_cell(&self, max_num: usize) -> String {
		if self.enabled == false || max_num == 0 {
			return String::new();
		}
		let width = max_num.to_string().len();
		" ".repeat(width)
	}
}
