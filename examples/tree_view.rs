use anyhow::Result;
use querypath::QueryPath;
use querypath_fmt::{Numeration, QueryPathFmt};

fn main() -> Result<()> {
    let res: querypath::QueryResults = QueryPath::new()
        .scan_at(["./"])
        .match_pattern(["!**/{.git|target}/?**"])
        .keep_parent(true)
        .run()?;

    println!("🔍 [querypath] Inicjalizacja skanowania...");
    println!(" ├─ Katalog roboczy (CWD): {}", res.execution_dir);
    println!(" ├─ Lokalizacje (scan_at): {:?}", res.scanned_paths);
    println!(" └─ Wzorce (match_pattern): {:?}", res.patterns);
    println!("📦 Zeskanowano fizycznie: {} plików, {} katalogów", res.scanned_files, res.scanned_dirs);

    println!("\n✨ Dopasowane katalogi ({}):", res.dirs.len());
    for d in &res.dirs {
        println!(
            " 📁 {} (dopasowane: {} B, pełne: {} B, mod: {:?})",
            d.path, d.matched_size, d.real_size, d.modified_at
        );
    }

    println!("\n✨ Dopasowane pliki ({}):", res.files.len());
    for f in &res.files {
        println!(" 📄 {} (rozmiar: {} B, binarny: {}, mod: {:?})", f.path, f.size, f.is_binary, f.modified_at);
    }

    println!("\n// a następnie użyjemy naszego querypath_fmt do:");

    let fmt = QueryPathFmt::new()
        .name_width(25)
        .path_width(45)
        .numeration(
            Numeration::new()
                .enabled(true)
                .numerate_dirs(false)
                .numerate_binaries(false)
                .start_from(1),
        );

    let output = fmt.format(&res);
    println!("{}", output);

    Ok(())
}