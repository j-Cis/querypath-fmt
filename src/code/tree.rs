/// Pełny system 17 symboli określających strukturę i wcięcia
pub const TREE_SYMBOLS: [&str; 17] = [
    "└───", "└──┬", "└──•", "   ", "├───", "├──┬", "├──•", "│  ",
    "  ├───", "  └───", "    •", "  ├──•", "  ┌──•", "  └──•", "   ──•",
    "▣─┬", "▣───"
];

#[derive(Debug, Clone)]
pub enum TreeItem {
    Root {
        label: String,
        path: String,
        children: Vec<TreeItem>,
    },
    Node {
        label: String,
        is_dir: bool,
        is_binary: bool,
        path: String,
        children: Vec<TreeItem>,
    },
    Spacer,
}

pub struct TreeRow {
    pub tree_line: String,
    pub path: String,
    pub is_dir: bool,
    pub is_binary: bool,
    pub is_first_line: bool,
}

pub struct Tree {
    pub root_items: Vec<TreeItem>,
    pub name_width: usize,
}

impl Tree {
    pub fn new() -> Self {
        Self {
            root_items: Vec::new(),
            name_width: 20,
        }
    }

    pub fn name_width(mut self, width: usize) -> Self {
        self.name_width = width.max(20);
        self
    }

    pub fn add_item(&mut self, item: TreeItem) {
        self.root_items.push(item);
    }

    pub fn calculate_max_prefix_len(&self) -> usize {
        let mut max_len = 0;
        for item in &self.root_items {
            self.walk_prefix_len(item, "", &mut max_len);
        }
        max_len
    }

    fn walk_prefix_len(&self, item: &TreeItem, prefix: &str, max_len: &mut usize) {
        match item {
            TreeItem::Spacer => {
                *max_len = (*max_len).max(prefix.chars().count());
            }
            TreeItem::Root { children, .. } => {
                *max_len = (*max_len).max(3); // "▣─┬"
                for child in children {
                    self.walk_prefix_len(child, "  ", max_len);
                }
            }
            TreeItem::Node { children, .. } => {
                let symbol_len = 4; // np. "├──┬"
                let line_prefix_len = prefix.chars().count() + symbol_len;
                *max_len = (*max_len).max(line_prefix_len);

                let child_prefix = format!("{}{}", prefix, TREE_SYMBOLS[7]);
                for child in children {
                    self.walk_prefix_len(child, &child_prefix, max_len);
                }
            }
        }
    }

    pub fn render_rows(&self) -> Vec<TreeRow> {
        let effective_name_width = self.name_width.max(20);
        let max_prefix_len = self.calculate_max_prefix_len();
        let total_col_width = max_prefix_len + 1 + effective_name_width;

        let mut rows = Vec::new();
        let total = self.root_items.len();

        for (i, item) in self.root_items.iter().enumerate() {
            let is_last = i == total - 1;
            self.render_item(item, "", is_last, effective_name_width, total_col_width, &mut rows);
        }

        rows
    }

    fn render_item(
        &self,
        item: &TreeItem,
        prefix: &str,
        is_last: bool,
        name_width: usize,
        total_col_width: usize,
        out: &mut Vec<TreeRow>,
    ) {
        match item {
            TreeItem::Spacer => {
                let line = format!("{:width$}", prefix, width = total_col_width);
                out.push(TreeRow {
                    tree_line: line,
                    path: String::new(),
                    is_dir: false,
                    is_binary: false,
                    is_first_line: true,
                });
            }
            TreeItem::Root { label, path, children } => {
                let has_children = !children.is_empty();
                let symbol = if has_children { TREE_SYMBOLS[15] } else { TREE_SYMBOLS[16] };

                let line_prefix = format!("{}{}", prefix, symbol);
                let cont_prefix: String = line_prefix
                    .chars()
                    .map(|c| if matches!(c, '│' | '├' | '┬' | '┌') { '│' } else { ' ' })
                    .collect();

                let name_chunks = self.chunk_str(label, name_width);

                for (i, chunk) in name_chunks.iter().enumerate() {
                    let pfx = if i == 0 { &line_prefix } else { &cont_prefix };
                    let left_str = format!("{} {}", pfx, chunk);
                    let line = format!("{:width$}", left_str, width = total_col_width);

                    out.push(TreeRow {
                        tree_line: line,
                        path: if i == 0 { path.clone() } else { String::new() },
                        is_dir: true,
                        is_binary: false,
                        is_first_line: i == 0,
                    });
                }

                let child_total = children.len();
                for (i, child) in children.iter().enumerate() {
                    let child_is_last = i == child_total - 1;
                    self.render_item(child, "  ", child_is_last, name_width, total_col_width, out);
                }
            }
            TreeItem::Node { label, is_dir, is_binary, path, children } => {
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
                    .map(|c| if matches!(c, '│' | '├' | '┬' | '┌') { '│' } else { ' ' })
                    .collect();

                let name_chunks = self.chunk_str(label, name_width);

                for (i, chunk) in name_chunks.iter().enumerate() {
                    let pfx = if i == 0 { &line_prefix } else { &cont_prefix };
                    let left_str = format!("{} {}", pfx, chunk);
                    let line = format!("{:width$}", left_str, width = total_col_width);

                    out.push(TreeRow {
                        tree_line: line,
                        path: if i == 0 { path.clone() } else { String::new() },
                        is_dir: *is_dir,
                        is_binary: *is_binary,
                        is_first_line: i == 0,
                    });
                }

                let child_prefix = if is_last {
                    format!("{}{}", prefix, TREE_SYMBOLS[3])
                } else {
                    format!("{}{}", prefix, TREE_SYMBOLS[7])
                };

                let child_total = children.len();
                for (i, child) in children.iter().enumerate() {
                    let child_is_last = i == child_total - 1;
                    self.render_item(child, &child_prefix, child_is_last, name_width, total_col_width, out);
                }
            }
        }
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