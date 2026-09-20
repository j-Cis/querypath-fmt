use std::collections::BTreeMap;
use querypath::QueryResults;

/// Pełny system 15 symboli określających strukturę i wcięcia
pub const TREE_SYMBOLS: [&str; 15] = [
    "└───", "└──┬", "└──•", "   ", "├───", "├──┬", "├──•", "│  ",
    "  ├───", "  └───", "    •", "  ├──•", "  ┌──•", "  └──•", "   ──•"
];

#[derive(Default)]
struct TreeNode {
    children: BTreeMap<String, TreeNode>,
    is_dir: bool,
    original_path: String,
}

pub struct PathsTree {
    column_width: usize,
    max_name_len: Option<usize>,
}

impl PathsTree {
    pub fn new() -> Self {
        Self {
            column_width: 35,
            max_name_len: None,
        }
    }

    /// Szerokość lewej kolumny drzewa.
    pub fn column_width(mut self, width: usize) -> Self {
        self.column_width = width;
        self
    }

    /// Maksymalna długość nazwy w linii. Dłuższe nazwy zostaną przełamane do nowej linii.
    pub fn max_name_len(mut self, max_len: usize) -> Self {
        self.max_name_len = Some(max_len);
        self
    }

    pub fn set_column_width(&mut self, width: usize) -> &mut Self {
        self.column_width = width;
        self
    }

    pub fn set_max_name_len(&mut self, max_len: Option<usize>) -> &mut Self {
        self.max_name_len = max_len;
        self
    }

    pub fn format_results(&self, res: &QueryResults) -> String {
        let mut root = TreeNode {
            children: BTreeMap::new(),
            is_dir: true,
            original_path: String::new(),
        };

        for dir in &res.dirs {
            self.insert_path(&mut root, &dir.path, true);
        }
        for file in &res.files {
            self.insert_path(&mut root, &file.path, false);
        }

        let mut output = String::new();
        let children_count = root.children.len();

        for (i, (name, node)) in root.children.iter().enumerate() {
            let is_last = i == children_count - 1;
            self.print_node(node, name, "", is_last, &mut output);
        }

        output
    }

    fn insert_path(&self, root: &mut TreeNode, path: &str, is_dir: bool) {
        let normalized = path.replace('\\', "/");
        let parts: Vec<&str> = normalized.split('/').filter(|s| !s.is_empty() && *s != ".").collect();

        if parts.is_empty() { return; }

        let mut current = root;
        let len = parts.len();

        for (i, part) in parts.into_iter().enumerate() {
            let is_last_part = i == len - 1;
            let current_is_dir = if is_last_part { is_dir } else { true };

            let node = current.children.entry(part.to_string()).or_insert_with(|| TreeNode {
                children: BTreeMap::new(),
                is_dir: current_is_dir,
                original_path: String::new(),
            });

            if is_last_part {
                let mut formatted_path = path.to_string();
                if is_dir && !formatted_path.ends_with('/') {
                    formatted_path.push('/');
                }
                node.original_path = formatted_path;
            }
            current = node;
        }
    }

    fn print_node(&self, node: &TreeNode, name: &str, prefix: &str, is_last: bool, out: &mut String) {
        let has_children = !node.children.is_empty();

        let symbol = match (node.is_dir, has_children, is_last) {
            (true, true, true) => TREE_SYMBOLS[1],   // "└──┬"
            (true, true, false) => TREE_SYMBOLS[5],  // "├──┬"
            (true, false, true) => TREE_SYMBOLS[0],  // "└───"
            (true, false, false) => TREE_SYMBOLS[4], // "├───"
            (false, _, true) => TREE_SYMBOLS[2],     // "└──•"
            (false, _, false) => TREE_SYMBOLS[6],    // "├──•"
        };

        let line_prefix = format!("{}{}", prefix, symbol);

        // Zachowujemy kreskę pionową `│` dla kolejnych linii złamanej nazwy
        let cont_prefix: String = line_prefix
            .chars()
            .map(|c| if c == '│' { '│' } else { ' ' })
            .collect();

        let display_path = if node.original_path.is_empty() {
            String::new()
        } else {
            node.original_path.clone()
        };

        let (name_chunks, path_chunks) = self.split_chunks(name, &display_path);
        let max_lines = name_chunks.len().max(path_chunks.len());

        for i in 0..max_lines {
            let pfx = if i == 0 { &line_prefix } else { &cont_prefix };
            let nm = name_chunks.get(i).map(|s| s.as_str()).unwrap_or("");
            let pth = path_chunks.get(i).map(|s| s.as_str()).unwrap_or("");

            let left_str = format!("{} {}", pfx, nm);
            let left_len = left_str.chars().count();

            let padding_len = self.column_width.saturating_sub(left_len);
            let padding = " ".repeat(padding_len);

            out.push_str(&format!("{}{}{}\n", left_str, padding, pth));
        }

        let child_prefix = if is_last {
            format!("{}{}", prefix, TREE_SYMBOLS[3]) // "   "
        } else {
            format!("{}{}", prefix, TREE_SYMBOLS[7]) // "│  "
        };

        let children_count = node.children.len();
        for (i, (child_name, child_node)) in node.children.iter().enumerate() {
            let child_is_last = i == children_count - 1;
            self.print_node(child_node, child_name, &child_prefix, child_is_last, out);
        }
    }

    fn split_chunks(&self, name: &str, full_path: &str) -> (Vec<String>, Vec<String>) {
        let max_len = match self.max_name_len {
            Some(limit) if limit > 0 => limit,
            _ => return (vec![name.to_string()], vec![full_path.to_string()]),
        };

        let name_chunks = self.chunk_str(name, max_len);

        let path_chunks = if !full_path.is_empty() && !full_path.ends_with('/') && full_path.ends_with(name) {
            let dir_part = &full_path[..full_path.len() - name.len()];
            let mut chunks = vec![format!("{}{}", dir_part, name_chunks[0])];
            for nc in name_chunks.iter().skip(1) {
                chunks.push(nc.clone());
            }
            chunks
        } else {
            self.chunk_str(full_path, max_len)
        };

        (name_chunks, path_chunks)
    }

    fn chunk_str(&self, s: &str, limit: usize) -> Vec<String> {
        if s.is_empty() {
            return vec![String::new()];
        }
        let chars: Vec<char> = s.chars().collect();
        chars
            .chunks(limit)
            .map(|chunk| chunk.iter().collect())
            .collect()
    }
}