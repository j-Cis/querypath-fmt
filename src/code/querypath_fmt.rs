use std::collections::BTreeMap;
use std::path::Path;
use querypath::QueryResults;

use crate::numeration::Numeration;
use crate::paths_tree::PathsTree;
use crate::tree::{Tree, TreeItem, TREE_SYMBOLS};

#[derive(Default)]
struct InternalNode {
    children: BTreeMap<String, InternalNode>,
    is_dir: bool,
    is_binary: bool,
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
            is_binary: self.is_binary,
            path: self.original_path.clone(),
            children,
        }
    }
}

pub struct QueryPathFmt {
    name_width: usize,
    path_width: usize,
    numeration: Numeration,
}

impl QueryPathFmt {
    pub fn new() -> Self {
        Self {
            name_width: 20,
            path_width: 40,
            numeration: Numeration::default(),
        }
    }

    pub fn name_width(mut self, width: usize) -> Self {
        self.name_width = width.max(20);
        self
    }

    pub fn path_width(mut self, width: usize) -> Self {
        self.path_width = width.max(40);
        self
    }

    pub fn numeration(mut self, numeration: Numeration) -> Self {
        self.numeration = numeration;
        self
    }

    pub fn format(&self, res: &QueryResults) -> String {
        let mut root = InternalNode {
            children: BTreeMap::new(),
            is_dir: true,
            is_binary: false,
            original_path: String::new(),
        };

        for dir in &res.dirs {
            self.insert_path(&mut root, &dir.path, true, false);
        }
        for file in &res.files {
            self.insert_path(&mut root, &file.path, false, file.is_binary);
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

        let root_node = TreeItem::Root {
            label: root_name,
            path: root_path,
            children: children_items,
        };

        let mut tree = Tree::new().name_width(self.name_width);
        tree.add_item(root_node);

        let paths_fmt = PathsTree::new().path_width(self.path_width);

        let mut total_numerated = 0;
        self.count_numerated(&tree.root_items, &mut total_numerated);

        let max_num = if total_numerated == 0 {
            0
        } else {
            self.numeration.start_from + total_numerated - 1
        };

        let max_prefix_len = tree.calculate_max_prefix_len();
        let total_tree_col_width = max_prefix_len + 1 + self.name_width.max(20);

        let mut output = String::new();
        let mut counter = self.numeration.start_from;

        for item in &tree.root_items {
            self.render_item_combined(
                item,
                "",
                true,
                total_tree_col_width,
                max_num,
                &paths_fmt,
                &mut counter,
                &mut output,
            );
        }

        output
    }

    fn count_numerated(&self, items: &[TreeItem], count: &mut usize) {
        for item in items {
            match item {
                TreeItem::Root { children, .. } => {
                    // Korzeń nie podlega numeracji, zliczamy tylko jego dzieci
                    self.count_numerated(children, count);
                }
                TreeItem::Node { is_dir, is_binary, children, .. } => {
                    if self.numeration.should_numerate(*is_dir, *is_binary) {
                        *count += 1;
                    }
                    self.count_numerated(children, count);
                }
                TreeItem::Spacer => {}
            }
        }
    }

    fn render_item_combined(
        &self,
        item: &TreeItem,
        prefix: &str,
        is_last: bool,
        total_tree_col_width: usize,
        max_num: usize,
        paths_fmt: &PathsTree,
        counter: &mut usize,
        out: &mut String,
    ) {
        let eff_name_width = self.name_width.max(20);

        match item {
            TreeItem::Spacer => {
                let empty_num = self.numeration.empty_cell(max_num);
                let line = format!("{}{:width$}\n", empty_num, prefix, width = total_tree_col_width);
                out.push_str(&line);
            }
            TreeItem::Root { label, path, children } => {
                let has_children = !children.is_empty();
                let symbol = if has_children { TREE_SYMBOLS[15] } else { TREE_SYMBOLS[16] };

                let line_prefix = format!("{}{}", prefix, symbol);
                let cont_prefix: String = line_prefix
                    .chars()
                    .map(|c| if matches!(c, '│' | '├' | '┬' | '┌') { '│' } else { ' ' })
                    .collect();

                // Korzeń dostaje pustą komórkę numeryczną (bez zwiększania countera)
                let num_str = self.numeration.empty_cell(max_num);

                let name_chunks = self.chunk_str(label, eff_name_width);
                let path_chunks = paths_fmt.format_path_chunks(path);
                let max_lines = name_chunks.len().max(path_chunks.len());

                for i in 0..max_lines {
                    let n_cell = &num_str;
                    let pfx = if i == 0 { &line_prefix } else { &cont_prefix };
                    let nm = name_chunks.get(i).map(|s| s.as_str()).unwrap_or("");
                    let pth = path_chunks.get(i).map(|s| s.as_str()).unwrap_or("");

                    let left_str = format!("{} {}", pfx, nm);
                    let formatted_tree_col = format!("{:width$}", left_str, width = total_tree_col_width);

                    out.push_str(n_cell);
                    out.push_str(&formatted_tree_col);
                    out.push_str(pth);
                    out.push('\n');
                }

                let child_total = children.len();
                for (i, child) in children.iter().enumerate() {
                    let child_is_last = i == child_total - 1;
                    self.render_item_combined(child, "  ", child_is_last, total_tree_col_width, max_num, paths_fmt, counter, out);
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

                let num_str = if self.numeration.should_numerate(*is_dir, *is_binary) {
                    let cell = self.numeration.format_cell(*counter, max_num);
                    *counter += 1;
                    cell
                } else {
                    self.numeration.empty_cell(max_num)
                };

                let name_chunks = self.chunk_str(label, eff_name_width);
                let path_chunks = paths_fmt.format_path_chunks(path);
                let max_lines = name_chunks.len().max(path_chunks.len());

                for i in 0..max_lines {
                    let n_cell = if i == 0 { &num_str } else { &self.numeration.empty_cell(max_num) };
                    let pfx = if i == 0 { &line_prefix } else { &cont_prefix };
                    let nm = name_chunks.get(i).map(|s| s.as_str()).unwrap_or("");
                    let pth = path_chunks.get(i).map(|s| s.as_str()).unwrap_or("");

                    let left_str = format!("{} {}", pfx, nm);
                    let formatted_tree_col = format!("{:width$}", left_str, width = total_tree_col_width);

                    out.push_str(n_cell);
                    out.push_str(&formatted_tree_col);
                    out.push_str(pth);
                    out.push('\n');
                }

                let child_prefix = if is_last {
                    format!("{}{}", prefix, TREE_SYMBOLS[3])
                } else {
                    format!("{}{}", prefix, TREE_SYMBOLS[7])
                };

                let child_total = children.len();
                for (i, child) in children.iter().enumerate() {
                    let child_is_last = i == child_total - 1;
                    self.render_item_combined(child, &child_prefix, child_is_last, total_tree_col_width, max_num, paths_fmt, counter, out);
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

    fn insert_path(&self, root: &mut InternalNode, path: &str, is_dir: bool, is_binary: bool) {
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
                });

            if is_last_part {
                let mut formatted_path = path.to_string();
                if is_dir && !formatted_path.ends_with('/') {
                    formatted_path.push('/');
                }
                node.original_path = formatted_path;
                node.is_binary = is_binary;
            }
            current = node;
        }
    }
}