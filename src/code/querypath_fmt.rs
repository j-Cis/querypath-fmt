use querypath::QueryResults;

use crate::fmt_node::InternalNode;
use crate::fmt_render::FmtRender;
use crate::numeration::Numeration;
use crate::stats_temporal::StatsTemporal;
use crate::stats_weight::StatsWeight;
use crate::tree::Tree;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Column {
    Weight,
    Temporal,
    Path,
}

pub struct QueryPathFmt {
    name_width: usize,
    path_width: usize,
    left_columns: Vec<Column>,
    right_columns: Vec<Column>,
    numeration: Numeration,
    stats_weight: StatsWeight,
    stats_temporal: StatsTemporal,
}

impl QueryPathFmt {
    pub fn new() -> Self {
        Self {
            name_width: 20,
            path_width: 40,
            left_columns: vec![Column::Weight, Column::Temporal],
            right_columns: vec![Column::Path],
            numeration: Numeration::default(),
            stats_weight: StatsWeight::default(),
            stats_temporal: StatsTemporal::default(),
        }
    }

    pub fn name_width(mut self, width: usize) -> Self {
        self.name_width = width.max(20);
        self
    }

    pub fn path_width(mut self, width: usize) -> Self {
        self.path_width = width.max(30);
        self
    }

    pub fn column_order_left(mut self, cols: impl IntoIterator<Item = Column>) -> Self {
        self.left_columns = cols.into_iter().collect();
        self
    }

    pub fn column_order_right(mut self, cols: impl IntoIterator<Item = Column>) -> Self {
        self.right_columns = cols.into_iter().collect();
        self
    }

    pub fn numeration(mut self, numeration: Numeration) -> Self {
        self.numeration = numeration;
        self
    }

    pub fn stats_weight(mut self, weight: StatsWeight) -> Self {
        self.stats_weight = weight;
        self
    }

    pub fn stats_temporal(mut self, temporal: StatsTemporal) -> Self {
        self.stats_temporal = temporal;
        self
    }

    pub fn format(&self, res: &QueryResults) -> String {
        let root_node = InternalNode::build_root(res);

        let tree = Tree::new().name_width(self.name_width);
        let max_prefix_len = tree.calculate_max_prefix_len_for_item(&root_node);

        let renderer = FmtRender {
            name_width: self.name_width,
            path_width: self.path_width,
            left_columns: &self.left_columns,
            right_columns: &self.right_columns,
            numeration: &self.numeration,
            stats_weight: &self.stats_weight,
            stats_temporal: &self.stats_temporal,
        };

        renderer.render(&root_node, max_prefix_len)
    }
}