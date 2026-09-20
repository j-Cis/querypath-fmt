#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnitSystem {
    Binary,  // IEC: B, KiB, MiB, GiB, TiB (dzielnik 1024)
    Decimal, // SI:  B, KB, MB, GB, TB   (dzielnik 1000)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WeightPrecision {
    Tenths,     // Dokładność do części dziesiętnych (np. 12.5 MiB)
    Hundredths, // Dokładność do części setnych (np. 12.54 MiB)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DirWeightDisplay {
    None,
    MatchedOnly,
    RealOnly,
    Both, // np. "12.5 MiB / 140.2 MiB"
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StatsWeight {
    pub enabled: bool,
    pub unit_system: UnitSystem,
    pub precision: WeightPrecision,
    pub dir_display: DirWeightDisplay,
    pub include_files: bool,
    pub include_binaries_in_matched: bool,
}

impl Default for StatsWeight {
    fn default() -> Self {
        Self {
            enabled: true,
            unit_system: UnitSystem::Binary,
            precision: WeightPrecision::Tenths,
            dir_display: DirWeightDisplay::MatchedOnly,
            include_files: true,
            include_binaries_in_matched: true,
        }
    }
}

impl StatsWeight {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn unit_system(mut self, system: UnitSystem) -> Self {
        self.unit_system = system;
        self
    }

    pub fn precision(mut self, precision: WeightPrecision) -> Self {
        self.precision = precision;
        self
    }

    pub fn dir_display(mut self, display: DirWeightDisplay) -> Self {
        self.dir_display = display;
        self
    }

    pub fn include_files(mut self, include: bool) -> Self {
        self.include_files = include;
        self
    }

    pub fn include_binaries_in_matched(mut self, include: bool) -> Self {
        self.include_binaries_in_matched = include;
        self
    }

    /// Formatuje pojedynczą wartość w bajtach na czytelny ciąg znaków.
    pub fn format_bytes(&self, bytes: u64) -> String {
        let (base, units): (f64, &[&str]) = match self.unit_system {
            UnitSystem::Binary => (1024.0, &["B", "KiB", "MiB", "GiB", "TiB"]),
            UnitSystem::Decimal => (1000.0, &["B", "KB", "MB", "GB", "TB"]),
        };

        if bytes == 0 {
            return format!("0 {}", units[0]);
        }

        let bytes_f = bytes as f64;
        let i = (bytes_f.ln() / base.ln()).floor() as usize;
        let i = i.min(units.len() - 1);

        if i == 0 {
            return format!("{} B", bytes);
        }

        let val = bytes_f / base.powi(i as i32);
        match self.precision {
            WeightPrecision::Tenths => format!("{:.1} {}", val, units[i]),
            WeightPrecision::Hundredths => format!("{:.2} {}", val, units[i]),
        }
    }

    /// Surowy tekst rozmiaru pliku.
    pub fn format_file_size_raw(&self, size: u64, is_binary: bool) -> String {
        if !self.enabled || !self.include_files {
            return String::new();
        }
        if is_binary && !self.include_binaries_in_matched {
            return String::new();
        }
        self.format_bytes(size)
    }

    /// Surowy tekst rozmiaru katalogu.
    pub fn format_dir_size_raw(&self, matched_size: u64, real_size: u64) -> String {
        if !self.enabled {
            return String::new();
        }

        match self.dir_display {
            DirWeightDisplay::None => String::new(),
            DirWeightDisplay::MatchedOnly => self.format_bytes(matched_size),
            DirWeightDisplay::RealOnly => self.format_bytes(real_size),
            DirWeightDisplay::Both => {
                let m = self.format_bytes(matched_size);
                let r = self.format_bytes(real_size);
                format!("{} / {}", m, r)
            }
        }
    }
}