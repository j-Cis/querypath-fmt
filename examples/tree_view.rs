use anyhow::Result;
use querypath::QueryPath;
use querypath_fmt::PathsTree;

fn main() -> Result<()> {
    // Odpytujemy własny silnik katalogów
    let res = QueryPath::new()
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
    
    // Przekazujemy QueryResults do wizualizatora
    let tree = PathsTree::new().format_results(&res);
    print!("{}", tree);

    Ok(())
}