# querypath-fmt

[![Crates.io](https://img.shields.io/crates/v/querypath-fmt.svg)](https://crates.io/crates/querypath-fmt)
[![Docs.rs](https://docs.rs/querypath-fmt/badge.svg)](https://docs.rs/querypath-fmt)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Unlicense-blue.svg)](LICENSE)

Advanced formatting, sorting, and rendering engine for:

[![Crates.io](https://img.shields.io/crates/v/querypath.svg)](https://crates.io/crates/querypath)
[![Docs.rs](https://docs.rs/querypath/badge.svg)](https://docs.rs/querypath)

`querypath-fmt` takes the raw scanning results from `querypath` and turns them into beautiful, highly customizable tree views. It features an advanced sorting engine, flexible column arrangements, and precise formatting for file sizes and dates.

## Features

- **Flexible Column Layout:** Arrange metadata columns (Path, Weight, Temporal) on either the left or right side of the immovable structural tree core.
- **Advanced Sorting Engine:** Sort nodes by custom group strategies (e.g., binaries first, then directories, then text files), extension priorities, character class precedence, and same-name conflict rules.
- **Weight Formatting:** Supports both Binary (KiB, MiB) and Decimal (KB, MB) unit systems, with configurable precision and directory size displays (matched vs. real).
- **Temporal Formatting:** Powered by `temporal-fmt`, offering advanced date tokens including astronomical and meteorological seasons.
- **Smart Numeration:** Optional dynamic numbering for directories and files aligned neatly with the tree structure.

## Usage

```rust
use anyhow::Result;
use querypath::QueryPath;
use querypath_fmt::{
    CasePrecedence, CharClass, Column, DirWeightDisplay, GroupStrategy, NoExtPriority, NodeGroup,
    Numeration, QueryPathFmt, SameNamePriority, Sorting, StatsTemporal, StatsWeight, UnitSystem,
    WeightPrecision,
};

fn main() -> Result<()> {
    // 1. Scan the filesystem using querypath
    let res = QueryPath::new()
        .scan_at(["./"])
        .match_pattern(["!**/{.git|target}/?**"])
        .keep_parent(true)
        .run()?;

    // 2. Format the results using querypath-fmt
    let fmt = QueryPathFmt::new()
        .name_width(25)
        .path_width(35)
        .column_order_left([])
        .column_order_right([
            Column::Weight,
            Column::Temporal,
            Column::Path,
        ])
        .numeration(Numeration::new().enabled(true))
        .stats_weight(
            StatsWeight::new()
                .unit_system(UnitSystem::Binary)
                .dir_display(DirWeightDisplay::MatchedOnly)
                .precision(WeightPrecision::Tenths),
        )
        .stats_temporal(
            StatsTemporal::new().pattern("YYYY-MM-MD AAA Q")
        )
        .sorting(
            Sorting::new()
                .group_strategy(GroupStrategy::Custom(vec![
                    NodeGroup::BinaryFile,
                    NodeGroup::Directory,
                    NodeGroup::TextFile,
                ]))
                .same_name_priority(SameNamePriority::DirectoryFirst)
                .no_ext_priority(NoExtPriority::Above)
                .ignore_leading_dot(true)
                .char_class_order([CharClass::Special, CharClass::Digit, CharClass::Letter])
                .case_precedence(CasePrecedence::Insensitive),
        );

    // 3. Render and print
    println!("{}", fmt.format(&res));

    Ok(())
}
```
