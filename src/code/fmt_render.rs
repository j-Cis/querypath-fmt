use crate::numeration::Numeration;
use crate::paths_tree::PathsTree;
use crate::querypathfmt::Column;
use crate::stats_temporal::StatsTemporal;
use crate::stats_weight::StatsWeight;
use crate::tree::{TreeItem, TREE_SYMBOLS};

pub struct FmtRender<'a> {
    pub name_width: usize,
    pub path_width: usize,
    pub left_columns: &'a [Column],
    pub right_columns: &'a [Column],
    pub numeration: &'a Numeration,
    pub stats_weight: &'a StatsWeight,
    pub stats_temporal: &'a StatsTemporal,
}

impl<'a> FmtRender<'a> {
    pub fn render(&self, root_item: &TreeItem, max_prefix_len: usize) -> String {
        let eff_name_width = self.name_width.max(20);
        let total_tree_col_width = max_prefix_len + 1 + eff_name_width;
        let paths_fmt = PathsTree::new().path_width(self.path_width);

        let mut total_numerated = 0;
        self.count_numerated(std::slice::from_ref(root_item), &mut total_numerated);

        let max_num = if total_numerated == 0 {
            0
        } else {
            self.numeration.start_from + total_numerated - 1
        };

        let mut max_weight_inner_len = 0;
        let mut max_temporal_len = 0;
        self.collect_max_widths(
            root_item,
            &mut max_weight_inner_len,
            &mut max_temporal_len,
        );

        let mut output = String::new();
        let mut counter = self.numeration.start_from;

        self.render_item_combined(
            root_item,
            "",
            true,
            total_tree_col_width,
            max_num,
            max_weight_inner_len,
            max_temporal_len,
            &paths_fmt,
            &mut counter,
            &mut output,
        );

        output
    }

    fn count_numerated(&self, items: &[TreeItem], count: &mut usize) {
        for item in items {
            match item {
                TreeItem::Root { children, .. } => {
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

    fn collect_max_widths(
        &self,
        item: &TreeItem,
        max_weight_inner_len: &mut usize,
        max_temporal_len: &mut usize,
    ) {
        match item {
            TreeItem::Spacer => {}
            TreeItem::Root { matched_size, real_size, modified_at, children, .. } => {
                let w_raw = self.stats_weight.format_dir_size_raw(*matched_size, *real_size);
                let t_str = self.stats_temporal.format_timestamp(*modified_at);
                *max_weight_inner_len = (*max_weight_inner_len).max(w_raw.chars().count());
                *max_temporal_len = (*max_temporal_len).max(t_str.chars().count());
                for child in children {
                    self.collect_max_widths(child, max_weight_inner_len, max_temporal_len);
                }
            }
            TreeItem::Node { is_dir, is_binary, size, matched_size, real_size, modified_at, children, .. } => {
                let w_raw = if *is_dir {
                    self.stats_weight.format_dir_size_raw(*matched_size, *real_size)
                } else {
                    self.stats_weight.format_file_size_raw(*size, *is_binary)
                };
                let t_str = self.stats_temporal.format_timestamp(*modified_at);
                *max_weight_inner_len = (*max_weight_inner_len).max(w_raw.chars().count());
                *max_temporal_len = (*max_temporal_len).max(t_str.chars().count());
                for child in children {
                    self.collect_max_widths(child, max_weight_inner_len, max_temporal_len);
                }
            }
        }
    }

    fn format_bracket_col(&self, raw_content: &str, max_inner_len: usize) -> String {
        if max_inner_len == 0 {
            String::new()
        } else if raw_content.is_empty() {
            " ".repeat(max_inner_len + 3)
        } else {
            format!("[{:>width$}] ", raw_content, width = max_inner_len)
        }
    }

    fn format_col(&self, content: &str, max_len: usize) -> String {
        if max_len == 0 {
            String::new()
        } else if content.is_empty() {
            " ".repeat(max_len + 1)
        } else {
            format!("{:>max_len$} ", content, max_len = max_len)
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn render_column_cell(
        &self,
        col: Column,
        is_first_line: bool,
        weight_raw: &str,
        temp_str: &str,
        path_chunk: &str,
        max_weight_inner_len: usize,
        max_temporal_len: usize,
    ) -> String {
        match col {
            Column::Weight => {
                if is_first_line {
                    self.format_bracket_col(weight_raw, max_weight_inner_len)
                } else {
                    self.format_bracket_col("", max_weight_inner_len)
                }
            }
            Column::Temporal => {
                if is_first_line {
                    self.format_col(temp_str, max_temporal_len)
                } else {
                    self.format_col("", max_temporal_len)
                }
            }
            Column::Path => {
                let eff_path_width = self.path_width.max(30);
                format!("{:width$} ", path_chunk, width = eff_path_width)
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn render_item_combined(
        &self,
        item: &TreeItem,
        prefix: &str,
        is_last: bool,
        total_tree_col_width: usize,
        max_num: usize,
        max_weight_inner_len: usize,
        max_temporal_len: usize,
        paths_fmt: &PathsTree,
        counter: &mut usize,
        out: &mut String,
    ) {
        let eff_name_width = self.name_width.max(20);

        match item {
            TreeItem::Spacer => {
                for col in self.left_columns {
                    out.push_str(&self.render_column_cell(*col, false, "", "", "", max_weight_inner_len, max_temporal_len));
                }
                out.push_str(&self.numeration.empty_cell(max_num));
                out.push_str(&format!("{:width$}", prefix, width = total_tree_col_width));
                for col in self.right_columns {
                    out.push_str(&self.render_column_cell(*col, false, "", "", "", max_weight_inner_len, max_temporal_len));
                }
                out.push('\n');
            }
            TreeItem::Root { label, path, matched_size, real_size, modified_at, children } => {
                let has_children = !children.is_empty();
                
                let symbol = if has_children {
                    TREE_SYMBOLS.get(15).copied().unwrap_or("")
                } else {
                    TREE_SYMBOLS.get(16).copied().unwrap_or("")
                };

                let line_prefix = format!("{}{}", prefix, symbol);
                let cont_prefix: String = line_prefix
                    .chars()
                    .map(|c| if matches!(c, '│' | '├' | '┬' | '┌') { '│' } else { ' ' })
                    .collect();

                let num_str = self.numeration.empty_cell(max_num);
                let weight_raw = self.stats_weight.format_dir_size_raw(*matched_size, *real_size);
                let temp_str = self.stats_temporal.format_timestamp(*modified_at);

                let name_chunks = self.chunk_str(label, eff_name_width);
                let path_chunks = paths_fmt.format_path_chunks(path);
                let max_lines = name_chunks.len().max(path_chunks.len());

                for i in 0..max_lines {
                    let is_first = i == 0;
                    let pfx = if is_first { &line_prefix } else { &cont_prefix };
                    let nm = name_chunks.get(i).map(|s| s.as_str()).unwrap_or("");
                    let pth = path_chunks.get(i).map(|s| s.as_str()).unwrap_or("");

                    let left_str = format!("{} {}", pfx, nm);
                    let formatted_tree_col = format!("{:width$}", left_str, width = total_tree_col_width);

                    for col in self.left_columns {
                        out.push_str(&self.render_column_cell(*col, is_first, &weight_raw, &temp_str, pth, max_weight_inner_len, max_temporal_len));
                    }

                    out.push_str(&num_str);
                    out.push_str(&formatted_tree_col);

                    for col in self.right_columns {
                        out.push_str(&self.render_column_cell(*col, is_first, &weight_raw, &temp_str, pth, max_weight_inner_len, max_temporal_len));
                    }

                    out.push('\n');
                }

                let child_total = children.len();
                for (i, child) in children.iter().enumerate() {
                    let child_is_last = i == child_total - 1;
                    self.render_item_combined(
                        child,
                        "  ",
                        child_is_last,
                        total_tree_col_width,
                        max_num,
                        max_weight_inner_len,
                        max_temporal_len,
                        paths_fmt,
                        counter,
                        out,
                    );
                }
            }
            TreeItem::Node { label, is_dir, is_binary, path, size, matched_size, real_size, modified_at, children } => {
                let has_children = !children.is_empty();

                let symbol = match (*is_dir, has_children, is_last) {
                    (true, true, true) => TREE_SYMBOLS.get(1).copied().unwrap_or(""),
                    (true, true, false) => TREE_SYMBOLS.get(5).copied().unwrap_or(""),
                    (true, false, true) => TREE_SYMBOLS.first().copied().unwrap_or(""),
                    (true, false, false) => TREE_SYMBOLS.get(4).copied().unwrap_or(""),
                    (false, _, true) => TREE_SYMBOLS.get(2).copied().unwrap_or(""),
                    (false, _, false) => TREE_SYMBOLS.get(6).copied().unwrap_or(""),
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

                let weight_raw = if *is_dir {
                    self.stats_weight.format_dir_size_raw(*matched_size, *real_size)
                } else {
                    self.stats_weight.format_file_size_raw(*size, *is_binary)
                };
                let temp_str = self.stats_temporal.format_timestamp(*modified_at);

                let name_chunks = self.chunk_str(label, eff_name_width);
                let path_chunks = paths_fmt.format_path_chunks(path);
                let max_lines = name_chunks.len().max(path_chunks.len());

                for i in 0..max_lines {
                    let is_first = i == 0;
                    let n_cell = if is_first { &num_str } else { &self.numeration.empty_cell(max_num) };

                    let pfx = if is_first { &line_prefix } else { &cont_prefix };
                    let nm = name_chunks.get(i).map(|s| s.as_str()).unwrap_or("");
                    let pth = path_chunks.get(i).map(|s| s.as_str()).unwrap_or("");

                    let left_str = format!("{} {}", pfx, nm);
                    let formatted_tree_col = format!("{:width$}", left_str, width = total_tree_col_width);

                    for col in self.left_columns {
                        out.push_str(&self.render_column_cell(*col, is_first, &weight_raw, &temp_str, pth, max_weight_inner_len, max_temporal_len));
                    }

                    out.push_str(n_cell);
                    out.push_str(&formatted_tree_col);

                    for col in self.right_columns {
                        out.push_str(&self.render_column_cell(*col, is_first, &weight_raw, &temp_str, pth, max_weight_inner_len, max_temporal_len));
                    }

                    out.push('\n');
                }

                let child_prefix = if is_last {
                    format!("{}{}", prefix, TREE_SYMBOLS.get(3).copied().unwrap_or(""))
                } else {
                    format!("{}{}", prefix, TREE_SYMBOLS.get(7).copied().unwrap_or(""))
                };

                let child_total = children.len();
                for (i, child) in children.iter().enumerate() {
                    let child_is_last = i == child_total - 1;
                    self.render_item_combined(
                        child,
                        &child_prefix,
                        child_is_last,
                        total_tree_col_width,
                        max_num,
                        max_weight_inner_len,
                        max_temporal_len,
                        paths_fmt,
                        counter,
                        out,
                    );
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