use std::collections::BTreeMap;
use std::path::Path;
use querypath::QueryResults;
use crate::tree::{Tree, TreeItem};

#[derive(Default)]
struct InternalNode {
    children: BTreeMap<String, InternalNode>,
    is_dir: bool,
    original_path: String,
}

impl InternalNode {
    fn to_tree_item(&self, label: String) -> TreeItem {
        let children = self
            .children
            .iter()
            .map(|(child_name, child_node)| child_node.to_tree_item(child_name.clone()))
            .collect();

        TreeItem::Node {
            label,
            is_dir: self.is_dir,
            path: self.original_path.clone(),
            children,
        }
    }
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

    /// Maksymalna długość nazwy w linii.
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
        let mut root = InternalNode {
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

        let root_name = Path::new(&res.execution_dir)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or(".")
            .to_string();

        // Używamy ścieżki absolutnej pobranej z res.execution_dir
        let mut root_path = res.execution_dir.replace('\\', "/");
        if !root_path.ends_with('/') {
            root_path.push('/');
        }

        let children_items: Vec<TreeItem> = root
            .children
            .into_iter()
            .map(|(child_name, child_node)| child_node.to_tree_item(child_name))
            .collect();

        let root_node = TreeItem::Root {
            label: root_name,
            path: root_path,
            children: children_items,
        };

        let mut tree = Tree::new().column_width(self.column_width);
        if let Some(limit) = self.max_name_len {
            tree = tree.max_name_len(limit);
        }

        tree.add_item(root_node);
        tree.render()
    }

    fn insert_path(&self, root: &mut InternalNode, path: &str, is_dir: bool) {
        let normalized = path.replace('\\', "/");
        let parts: Vec<&str> = normalized
            .split('/')
            .filter(|s| !s.is_empty() && *s != ".")
            .collect();

        if parts.is_empty() {
            return;
        }

        let mut current = root;
        let len = parts.len();

        for (i, part) in parts.into_iter().enumerate() {
            let is_last_part = i == len - 1;
            let current_is_dir = if is_last_part { is_dir } else { true };

            let node = current
                .children
                .entry(part.to_string())
                .or_insert_with(|| InternalNode {
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
}