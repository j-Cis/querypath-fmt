pub struct PathsTree {
    path_width: usize,
}

impl PathsTree {
    pub fn new() -> Self {
        Self { path_width: 40 }
    }

    pub fn path_width(mut self, width: usize) -> Self {
        self.path_width = width.max(40);
        self
    }

    pub fn format_path_chunks(&self, full_path: &str) -> Vec<String> {
        let width = self.path_width.max(40);
        if full_path.is_empty() {
            return vec![String::new()];
        }

        let chars: Vec<char> = full_path.chars().collect();
        chars
            .chunks(width)
            .map(|chunk| chunk.iter().collect())
            .collect()
    }
}