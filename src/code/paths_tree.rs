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
}

impl PathsTree {
    pub fn new() -> Self {
        Self {
            column_width: 35, // Zapewnia wyrównanie oryginalnych ścieżek do równej kolumny
        }
    }

    pub fn format_results(&self, res: &QueryResults) -> String {
        let mut root = TreeNode {
            children: BTreeMap::new(),
            is_dir: true,
            original_path: String::new(),
        };

        // Zasilamy drzewo strukturami z querypath
        for dir in &res.dirs {
            self.insert_path(&mut root, &dir.path, true);
        }
        for file in &res.files {
            self.insert_path(&mut root, &file.path, false);
        }

        let mut output = String::new();
        let children_count = root.children.len();
        
        // Iterujemy tylko po dzieciach roota, aby nie rysować nadrzędnego "." lub ""
        for (i, (name, node)) in root.children.iter().enumerate() {
            let is_last = i == children_count - 1;
            self.print_node(node, name, "", is_last, &mut output);
        }
        
        output
    }

    fn insert_path(&self, root: &mut TreeNode, path: &str, is_dir: bool) {
        // Normalizujemy ścieżki i omijamy nadrzędną kropkę (./)
        let normalized = path.replace('\\', "/");
        let parts: Vec<&str> = normalized.split('/').filter(|s| !s.is_empty() && *s != ".").collect();
        
        if parts.is_empty() { return; }

        let mut current = root;
        let len = parts.len();
        
        for (i, part) in parts.into_iter().enumerate() {
            let is_last_part = i == len - 1;
            // Węzły pośrednie muszą być katalogami
            let current_is_dir = if is_last_part { is_dir } else { true };
            
            let node = current.children.entry(part.to_string()).or_insert_with(|| TreeNode {
                children: BTreeMap::new(),
                is_dir: current_is_dir,
                original_path: String::new(),
            });
            
            if is_last_part {
                // Dodajemy ukośnik dla katalogów na końcu prezentowanej ścieżki po prawej
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
        
        // Dynamiczny dobór symbolu bazujący na typie pliku/katalogu z naszej 15-znakowej palety
        let symbol = match (node.is_dir, has_children, is_last) {
            (true, true, true) => TREE_SYMBOLS[1],   // "└──┬"
            (true, true, false) => TREE_SYMBOLS[5],  // "├──┬"
            (true, false, true) => TREE_SYMBOLS[0],  // "└───"
            (true, false, false) => TREE_SYMBOLS[4], // "├───"
            (false, _, true) => TREE_SYMBOLS[2],     // "└──•"
            (false, _, false) => TREE_SYMBOLS[6],    // "├──•"
        };

        let line_prefix = format!("{}{}", prefix, symbol);
        let tree_part = format!("{} {}", line_prefix, name);
        
        // Automatyczne wcięcie do prawej kolumny ze ścieżkami
        let padding_len = self.column_width.saturating_sub(tree_part.chars().count());
        let padding = " ".repeat(padding_len);

        let display_path = if node.original_path.is_empty() {
            "".to_string()
        } else {
            node.original_path.clone()
        };

        out.push_str(&format!("{}{}{}\n", tree_part, padding, display_path));

        // Obliczanie wcięcia dla dzieci
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
}