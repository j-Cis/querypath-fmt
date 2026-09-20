use std::collections::BTreeMap;
use std::path::Path;
use querypath::QueryResults;
use crate::tree::TreeItem;

#[derive(Default)]
pub struct InternalNode {
    pub children: BTreeMap<String, InternalNode>,
    pub is_dir: bool,
    pub is_binary: bool,
    pub original_path: String,
    pub size: u64,
    pub matched_size: u64,
    pub real_size: u64,
    pub modified_at: Option<u64>,
}

impl InternalNode {
    pub fn to_tree_item(&self, label: String) -> TreeItem {
        let children = self
            .children
            .iter()
            .map(|(child_name, child_node)| child_node.to_tree_item(child_name.clone()))
            .collect();

        TreeItem::Node {
            label,
            is_dir: self.is_dir,
            is_binary: self.is_binary,
            path: self.original_path.clone(),
            size: self.size,
            matched_size: self.matched_size,
            real_size: self.real_size,
            modified_at: self.modified_at,
            children,
        }
    }

    pub fn insert_path(
        root: &mut InternalNode,
        path: &str,
        is_dir: bool,
        is_binary: bool,
        size: u64,
        matched_size: u64,
        real_size: u64,
        modified_at: Option<u64>,
    ) {
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
            let current_is_bin = if is_last_part { is_binary } else { false };

            let node = current
                .children
                .entry(part.to_string())
                .or_insert_with(|| InternalNode {
                    children: BTreeMap::new(),
                    is_dir: current_is_dir,
                    is_binary: current_is_bin,
                    original_path: String::new(),
                    size: 0,
                    matched_size: 0,
                    real_size: 0,
                    modified_at: None,
                });

            if is_last_part {
                let mut formatted_path = path.to_string();
                if is_dir && !formatted_path.ends_with('/') {
                    formatted_path.push('/');
                }
                node.original_path = formatted_path;
                node.is_binary = is_binary;
                node.size = size;
                node.matched_size = matched_size;
                node.real_size = real_size;
                node.modified_at = modified_at;
            }
            current = node;
        }
    }

    pub fn build_root(res: &QueryResults) -> TreeItem {
        let mut root = InternalNode {
            children: BTreeMap::new(),
            is_dir: true,
            is_binary: false,
            original_path: String::new(),
            size: 0,
            matched_size: 0,
            real_size: 0,
            modified_at: None,
        };

        for dir in &res.dirs {
            Self::insert_path(
                &mut root,
                &dir.path,
                true,
                false,
                0,
                dir.matched_size,
                dir.real_size,
                dir.modified_at,
            );
        }
        for file in &res.files {
            Self::insert_path(
                &mut root,
                &file.path,
                false,
                file.is_binary,
                file.size,
                0,
                0,
                file.modified_at,
            );
        }

        let root_name = Path::new(&res.execution_dir)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or(".")
            .to_string();

        let mut root_path = res.execution_dir.replace('\\', "/");
        if !root_path.ends_with('/') {
            root_path.push('/');
        }

        let children_items: Vec<TreeItem> = root
            .children
            .into_iter()
            .map(|(child_name, child_node)| child_node.to_tree_item(child_name))
            .collect();

        let root_matched: u64 = res.files.iter().map(|f| f.size).sum();
        let root_real: u64 = res.dirs.iter().map(|d| d.real_size).sum();

        TreeItem::Root {
            label: root_name,
            path: root_path,
            matched_size: root_matched,
            real_size: root_real,
            modified_at: None,
            children: children_items,
        }
    }
}