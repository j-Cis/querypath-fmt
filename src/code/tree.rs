/// Pełny system 15 symboli określających strukturę i wcięcia
pub const TREE_SYMBOLS: [&str; 15] = [
    "└───", "└──┬", "└──•", "   ", "├───", "├──┬", "├──•", "│  ",
    "  ├───", "  └───", "    •", "  ├──•", "  ┌──•", "  └──•", "   ──•"
];

#[derive(Debug, Clone)]
pub enum TreeItem {
    Node {
        label: String,
        is_dir: bool,
        path: String,
        children: Vec<TreeItem>,
    },
    /// Odstęp wizualny kontynuujący kreski pionowe nadrzędnych gałęzi `│`
    Spacer,
}

pub struct Tree {
    pub root_items: Vec<TreeItem>,
    pub column_width: usize,
    pub max_name_len: Option<usize>,
}

impl Tree {
    pub fn new() -> Self {
        Self {
            root_items: Vec::new(),
            column_width: 35,
            max_name_len: None,
        }
    }

    pub fn column_width(mut self, width: usize) -> Self {
        self.column_width = width;
        self
    }

    pub fn max_name_len(mut self, max_len: usize) -> Self {
        self.max_name_len = Some(max_len);
        self
    }

    pub fn add_item(&mut self, item: TreeItem) {
        self.root_items.push(item);
    }

    pub fn render(&self) -> String {
        let mut output = String::new();
        let total = self.root_items.len();

        for (i, item) in self.root_items.iter().enumerate() {
            let is_last = i == total - 1;
            self.render_item(item, "", is_last, &mut output);
        }

        output
    }

    fn render_item(&self, item: &TreeItem, prefix: &str, is_last: bool, out: &mut String) {
        match item {
            TreeItem::Spacer => {
                out.push_str(prefix);
                out.push('\n');
            }
            TreeItem::Node { label, is_dir, path, children } => {
                let has_children = !children.is_empty();

                let symbol = match (*is_dir, has_children, is_last) {
                    (true, true, true) => TREE_SYMBOLS[1],   // "└──┬"
                    (true, true, false) => TREE_SYMBOLS[5],  // "├──┬"
                    (true, false, true) => TREE_SYMBOLS[0],  // "└───"
                    (true, false, false) => TREE_SYMBOLS[4], // "├───"
                    (false, _, true) => TREE_SYMBOLS[2],     // "└──•"
                    (false, _, false) => TREE_SYMBOLS[6],    // "├──•"
                };

                let line_prefix = format!("{}{}", prefix, symbol);

                let cont_prefix: String = line_prefix
                    .chars()
                    .map(|c| if c == '│' { '│' } else { ' ' })
                    .collect();

                let (name_chunks, path_chunks) = self.split_chunks(label, path);
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

                let child_total = children.len();
                for (i, child) in children.iter().enumerate() {
                    let child_is_last = i == child_total - 1;
                    self.render_item(child, &child_prefix, child_is_last, out);
                }
            }
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