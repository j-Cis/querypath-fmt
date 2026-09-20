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
        matched_size: u64,
        real_size: u64,
        modified_at: Option<u64>,
        children: Vec<TreeItem>,
    },
    Node {
        label: String,
        is_dir: bool,
        is_binary: bool,
        path: String,
        size: u64,
        matched_size: u64,
        real_size: u64,
        modified_at: Option<u64>,
        children: Vec<TreeItem>,
    },
    Spacer,
}

impl Default for Tree {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Tree {
    pub name_width: usize,
}

impl Tree {
    pub fn new() -> Self {
        Self { name_width: 25 } // <-- Domyślnie 25
    }

    pub fn name_width(mut self, width: usize) -> Self {
        self.name_width = width.max(15); // <-- Minimum 15
        self
    }

    pub fn calculate_max_prefix_len_for_item(&self, item: &TreeItem) -> usize {
        let mut max_len = 0;
        self.walk_prefix_len(item, "", &mut max_len);
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
}

// Akcesory używane przez silnik sortujący
impl TreeItem {
    pub fn is_dir(&self) -> bool {
        match self {
            TreeItem::Root { .. } => true,
            TreeItem::Node { is_dir, .. } => *is_dir,
            TreeItem::Spacer => false,
        }
    }

    pub fn label(&self) -> &str {
        match self {
            TreeItem::Root { label, .. } => label,
            TreeItem::Node { label, .. } => label,
            TreeItem::Spacer => "",
        }
    }
}