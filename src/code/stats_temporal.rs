use temporal_fmt::Temporal;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatsTemporal {
    pub enabled: bool,
    pub format_pattern: String,
}

impl Default for StatsTemporal {
    fn default() -> Self {
        Self {
            enabled: true,
            format_pattern: "YYYY-MM-MD hh:mm".to_string(),
        }
    }
}

impl StatsTemporal {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn pattern(mut self, pattern: impl Into<String>) -> Self {
        self.format_pattern = pattern.into();
        self
    }

    /// Formatuje opcjonalny znacznik czasu UNIX (w sekundach) i zamyka w nawiasach []
    pub fn format_timestamp(&self, timestamp: Option<u64>) -> String {
        if self.enabled == false {
            return String::new();
        }

        let ts = match timestamp {
            Some(t) => t,
            std::option::Option::None => return String::new(),
        };

        let formatted = Temporal::format(ts, &self.format_pattern);

        if formatted.is_empty() {
            String::new()
        } else {
            format!("[{}]", formatted)
        }
    }
}