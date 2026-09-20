pub struct Temporal;

impl Temporal {
    /// Formatuje znacznik czasu UNIX (w sekundach) według podanego wzorca.
    pub fn format(timestamp: u64, pattern: &str) -> String {
        let dt = DateTimeUtc::from_secs(timestamp);
        Self::replace_tokens(pattern, &dt)
    }

    fn replace_tokens(pattern: &str, dt: &DateTimeUtc) -> String {
    let mut i = 0;
    let chars: Vec<char> = pattern.chars().collect();
    let len = chars.len();
    let mut out = String::new();

    while i < len {
        let rest = &pattern[i..];

        if rest.starts_with("WYYY") { out.push_str(&format!("{:04}", dt.iso_year)); i += 4; }
        else if rest.starts_with("YYYY") { out.push_str(&format!("{:04}", dt.year)); i += 4; }
        else if rest.starts_with("hhhh") { out.push_str(&dt.hour_12_format()); i += 4; }
        else if rest.starts_with("AAA") { out.push_str(dt.astro_season_abbr()); i += 3; }
        else if rest.starts_with("SSS") { out.push_str(dt.meteo_season_abbr()); i += 3; }
        else if rest.starts_with("MMM") { out.push_str(dt.month_abbr()); i += 3; }
        else if rest.starts_with("YDD") { out.push_str(&format!("{:03}", dt.day_of_year)); i += 3; }
        else if rest.starts_with("DDD") { out.push_str(dt.weekday_abbr()); i += 3; }
        else if rest.starts_with("zzz") { out.push_str(&format!("{:03}", dt.millisecond)); i += 3; }
        else if rest.starts_with("WY") { out.push_str(&format!("{:02}", dt.iso_year % 100)); i += 2; }
        else if rest.starts_with("YY") { out.push_str(&format!("{:02}", dt.year % 100)); i += 2; }
        else if rest.starts_with("MM") { out.push_str(&format!("{:02}", dt.month)); i += 2; }
        else if rest.starts_with("MD") { out.push_str(&format!("{:02}", dt.day)); i += 2; }
        else if rest.starts_with("WW") { out.push_str(&format!("{:02}", dt.iso_week)); i += 2; }
        else if rest.starts_with("hh") { out.push_str(&format!("{:02}", dt.hour)); i += 2; }
        else if rest.starts_with("mm") { out.push_str(&format!("{:02}", dt.minute)); i += 2; }
        else if rest.starts_with("ss") { out.push_str(&format!("{:02}", dt.second)); i += 2; }
        else if rest.starts_with("tt") { out.push_str(&format!("{:02}", dt.tierce)); i += 2; }
        else if rest.starts_with("qq") { out.push_str(&format!("{:02}", dt.quadra)); i += 2; }
        else if rest.starts_with('A') { out.push_str(&dt.astro_season_num().to_string()); i += 1; }
        else if rest.starts_with('S') { out.push_str(&dt.meteo_season_num().to_string()); i += 1; }
        else if rest.starts_with('Q') { out.push_str(&dt.quarter().to_string()); i += 1; }
        else if rest.starts_with('D') { out.push_str(&dt.weekday_num().to_string()); i += 1; }
        else {
            out.push(chars[i]);
            i += 1;
        }
    }

    out
}
}

struct DateTimeUtc {
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
    minute: u32,
    second: u32,
    millisecond: u32,
    tierce: u32,
    quadra: u32,
    day_of_year: u32,
    weekday: u32,
    iso_week: u32,
    iso_year: i32,
}

impl DateTimeUtc {
    fn from_secs(secs: u64) -> Self {
        let days = (secs / 86400) as i64;
        let rem_secs = (secs % 86400) as u32;

        let hour = rem_secs / 3600;
        let minute = (rem_secs % 3600) / 60;
        let second = rem_secs % 60;

        let weekday = (((days + 3) % 7) + 1) as u32;

        let z = days + 719468;
        let era = (if z >= 0 { z } else { z - 146096 }) / 146097;
        let doe = (z - era * 146097) as u32;
        let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
        let y = yoe as i32 + era as i32 * 400;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp = (5 * doy + 2) / 153;
        let d = doy - (153 * mp + 2) / 5 + 1;
        let m = if mp < 10 { mp + 3 } else { mp - 9 };
        let year = if m <= 2 { y + 1 } else { y };

        let is_leap = (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0);
        let doy_calendar = if m > 2 {
            let month_days = [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334];
            month_days[(m - 1) as usize] + d + if is_leap { 1 } else { 0 }
        } else {
            let month_days = [0, 31];
            month_days[(m - 1) as usize] + d
        };

        let wday = weekday;
        let doy_i = doy_calendar as i32;
        let iso_w = (doy_i - wday as i32 + 10) / 7;
        let (iso_week, iso_year) = if iso_w < 1 {
            (52, year - 1)
        } else if iso_w > 52 {
            (1, year + 1)
        } else {
            (iso_w as u32, year)
        };

        Self {
            year,
            month: m,
            day: d,
            hour,
            minute,
            second,
            millisecond: 0,
            tierce: 0,
            quadra: 0,
            day_of_year: doy_calendar,
            weekday,
            iso_week,
            iso_year,
        }
    }

    fn weekday_num(&self) -> u32 {
        self.weekday
    }

    fn weekday_abbr(&self) -> &'static str {
        match self.weekday {
            1 => "MON",
            2 => "TUE",
            3 => "WED",
            4 => "THU",
            5 => "FRI",
            6 => "SAT",
            7 => "SUN",
            _ => "MON",
        }
    }

    fn month_abbr(&self) -> &'static str {
        match self.month {
            1 => "JAN",
            2 => "FEB",
            3 => "MAR",
            4 => "APR",
            5 => "MAY",
            6 => "JUN",
            7 => "JUL",
            8 => "AUG",
            9 => "SEP",
            10 => "OCT",
            11 => "NOV",
            12 => "DEC",
            _ => "JAN",
        }
    }

    fn hour_12_format(&self) -> String {
        let is_pm = self.hour >= 12;
        let h12 = match self.hour % 12 {
            0 => 12,
            h => h,
        };
        format!("{}{:02}", if is_pm { "pm" } else { "am" }, h12)
    }

    fn quarter(&self) -> u32 {
        ((self.month - 1) / 3) + 1
    }

    fn meteo_season_num(&self) -> u32 {
        match self.month {
            9 | 10 | 11 => 1,
            12 | 1 | 2 => 2,
            3 | 4 | 5 => 3,
            6 | 7 | 8 => 4,
            _ => 1,
        }
    }

    fn meteo_season_abbr(&self) -> &'static str {
        match self.meteo_season_num() {
            1 => "AUT",
            2 => "WIN",
            3 => "SPR",
            4 => "SUM",
            _ => "AUT",
        }
    }

    fn astro_season_num(&self) -> u32 {
        let m = self.month;
        let d = self.day;

        match (m, d) {
            (3, 21..=31) | (4, _) | (5, _) | (6, 1..=21) => 3,
            (6, 22..=31) | (7, _) | (8, _) | (9, 1..=22) => 4,
            (9, 23..=31) | (10, _) | (11, _) | (12, 1..=21) => 1,
            _ => 2,
        }
    }

    fn astro_season_abbr(&self) -> &'static str {
        match self.astro_season_num() {
            1 => "AUT",
            2 => "WIN",
            3 => "SPR",
            4 => "SUM",
            _ => "AUT",
        }
    }
}