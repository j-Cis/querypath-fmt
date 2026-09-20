use std::cmp::Ordering;
use crate::tree::TreeItem;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeGroup {
    Directory,
    TextFile,
    BinaryFile,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GroupStrategy {
    Mixed,
    Custom(Vec<NodeGroup>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SameNamePriority {
    DirectoryFirst,
    FileFirst,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoExtPriority {
    Above,
    Below,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CharClass {
    Special,
    Digit,
    Letter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CasePrecedence {
    Insensitive,
    UpperFirst,
    LowerFirst,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sorting {
    pub enabled: bool,
    pub group_strategy: GroupStrategy,
    pub same_name_priority: SameNamePriority,
    pub ignore_leading_dot: bool,
    pub split_extension: bool,
    pub no_ext_priority: NoExtPriority,
    pub char_class_order: [CharClass; 3],
    pub case_precedence: CasePrecedence,
    pub reverse: bool,
}

impl Default for Sorting {
    fn default() -> Self {
        Self {
            enabled: true,
            group_strategy: GroupStrategy::Mixed,
            same_name_priority: SameNamePriority::DirectoryFirst,
            ignore_leading_dot: false,
            split_extension: true,
            no_ext_priority: NoExtPriority::Above,
            char_class_order: [CharClass::Special, CharClass::Digit, CharClass::Letter],
            case_precedence: CasePrecedence::Insensitive,
            reverse: false,
        }
    }
}

impl Sorting {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn group_strategy(mut self, strategy: GroupStrategy) -> Self {
        self.group_strategy = strategy;
        self
    }

    pub fn same_name_priority(mut self, priority: SameNamePriority) -> Self {
        self.same_name_priority = priority;
        self
    }

    pub fn ignore_leading_dot(mut self, ignore: bool) -> Self {
        self.ignore_leading_dot = ignore;
        self
    }

    pub fn split_extension(mut self, split: bool) -> Self {
        self.split_extension = split;
        self
    }

    pub fn no_ext_priority(mut self, priority: NoExtPriority) -> Self {
        self.no_ext_priority = priority;
        self
    }

    pub fn char_class_order(mut self, order: [CharClass; 3]) -> Self {
        self.char_class_order = order;
        self
    }

    pub fn case_precedence(mut self, precedence: CasePrecedence) -> Self {
        self.case_precedence = precedence;
        self
    }

    pub fn reverse(mut self, reverse: bool) -> Self {
        self.reverse = reverse;
        self
    }

    pub fn sort_items(&self, items: &mut [TreeItem]) {
        if self.enabled == false || items.is_empty() {
            return;
        }

        items.sort_by(|a, b| {
            let res = self.compare_items(a, b);
            if self.reverse {
                res.reverse()
            } else {
                res
            }
        });

        for item in items {
            match item {
                TreeItem::Root { children, .. } | TreeItem::Node { children, .. } => {
                    self.sort_items(children);
                }
                TreeItem::Spacer => {}
            }
        }
    }

    fn compare_items(&self, a: &TreeItem, b: &TreeItem) -> Ordering {
        if let GroupStrategy::Custom(ref group_order) = self.group_strategy {
            let group_a = self.node_group(a);
            let group_b = self.node_group(b);

            if group_a != group_b {
                let pos_a = group_order.iter().position(|g| *g == group_a).unwrap_or(usize::MAX);
                let pos_b = group_order.iter().position(|g| *g == group_b).unwrap_or(usize::MAX);
                return pos_a.cmp(&pos_b);
            }
        }

        let label_a = a.label();
        let label_b = b.label();

        let clean_a = if self.ignore_leading_dot {
            label_a.strip_prefix('.').unwrap_or(label_a)
        } else {
            label_a
        };

        let clean_b = if self.ignore_leading_dot {
            label_b.strip_prefix('.').unwrap_or(label_b)
        } else {
            label_b
        };

        if self.split_extension {
            let (base_a, ext_a) = self.split_name_ext(clean_a, a.is_dir());
            let (base_b, ext_b) = self.split_name_ext(clean_b, b.is_dir());

            let base_cmp = self.compare_strings(&base_a, &base_b);
            if base_cmp != Ordering::Equal {
                return base_cmp;
            }

            let is_dir_a = a.is_dir();
            let is_dir_b = b.is_dir();

            if is_dir_a != is_dir_b {
                return match self.same_name_priority {
                    SameNamePriority::DirectoryFirst => {
                        if is_dir_a { Ordering::Less } else { Ordering::Greater }
                    }
                    SameNamePriority::FileFirst => {
                        if is_dir_a { Ordering::Greater } else { Ordering::Less }
                    }
                };
            }

            match (ext_a, ext_b) {
                (std::option::Option::None, std::option::Option::None) => Ordering::Equal,
                (std::option::Option::None, Some(_)) => match self.no_ext_priority {
                    NoExtPriority::Above => Ordering::Less,
                    NoExtPriority::Below => Ordering::Greater,
                },
                (Some(_), std::option::Option::None) => match self.no_ext_priority {
                    NoExtPriority::Above => Ordering::Greater,
                    NoExtPriority::Below => Ordering::Less,
                },
                (Some(e_a), Some(e_b)) => self.compare_strings(&e_a, &e_b),
            }
        } else {
            self.compare_strings(clean_a, clean_b)
        }
    }

    fn compare_strings(&self, s1: &str, s2: &str) -> Ordering {
        let chars1: Vec<char> = s1.chars().collect();
        let chars2: Vec<char> = s2.chars().collect();
        let min_len = chars1.len().min(chars2.len());

        for i in 0..min_len {
            let c1 = chars1[i];
            let c2 = chars2[i];

            if c1 == c2 {
                continue;
            }

            let class1 = self.classify_char(c1);
            let class2 = self.classify_char(c2);

            if class1 != class2 {
                let pos1 = self.char_class_order.iter().position(|c| *c == class1).unwrap_or(3);
                let pos2 = self.char_class_order.iter().position(|c| *c == class2).unwrap_or(3);
                return pos1.cmp(&pos2);
            }

            if class1 == CharClass::Letter {
                let lower1 = c1.to_lowercase().next().unwrap_or(c1);
                let lower2 = c2.to_lowercase().next().unwrap_or(c2);

                if lower1 != lower2 {
                    return lower1.cmp(&lower2);
                }

                match self.case_precedence {
                    CasePrecedence::Insensitive => continue,
                    CasePrecedence::UpperFirst => {
                        if c1.is_uppercase() && c2.is_lowercase() {
                            return Ordering::Less;
                        } else if c1.is_lowercase() && c2.is_uppercase() {
                            return Ordering::Greater;
                        }
                    }
                    CasePrecedence::LowerFirst => {
                        if c1.is_lowercase() && c2.is_uppercase() {
                            return Ordering::Less;
                        } else if c1.is_uppercase() && c2.is_lowercase() {
                            return Ordering::Greater;
                        }
                    }
                }
            } else {
                let char_cmp = c1.cmp(&c2);
                if char_cmp != Ordering::Equal {
                    return char_cmp;
                }
            }
        }

        chars1.len().cmp(&chars2.len())
    }

    fn classify_char(&self, c: char) -> CharClass {
        if c.is_ascii_digit() {
            CharClass::Digit
        } else if c.is_alphabetic() {
            CharClass::Letter
        } else {
            CharClass::Special
        }
    }

    fn split_name_ext(&self, name: &str, is_dir: bool) -> (String, Option<String>) {
        if is_dir {
            return (name.to_string(), None);
        }

        if let Some(pos) = name.rfind('.').filter(|&pos| pos > 0) {
            let (base, ext_with_dot) = name.split_at(pos);
            let ext = ext_with_dot.strip_prefix('.').unwrap_or(ext_with_dot).to_string();
            return (base.to_string(), Some(ext));
        }

        (name.to_string(), None)
    }

    fn node_group(&self, item: &TreeItem) -> NodeGroup {
        match item {
            TreeItem::Root { .. } => NodeGroup::Directory,
            TreeItem::Node { is_dir, is_binary, .. } => {
                if *is_dir {
                    NodeGroup::Directory
                } else if *is_binary {
                    NodeGroup::BinaryFile
                } else {
                    NodeGroup::TextFile
                }
            }
            TreeItem::Spacer => NodeGroup::TextFile,
        }
    }
}