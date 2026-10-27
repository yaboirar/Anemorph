//! Compile the user's episode format once, then reuse it for each file.
pub const DEFAULT_FORMAT: &str = "[Num][S][SeaNum][E][EpsNum]";

pub const DEFAULT_MOVIE_FORMAT: &str = "[Movie Name] Movie";

#[derive(Debug)]
enum MoviePart {
    Text(String),
    Name,
}
#[derive(Debug)]
pub struct MovieFormat {
    parts: Vec<MoviePart>,
}
impl MovieFormat {
    pub fn parse(pattern: &str) -> Result<Self, String> {
        if pattern.len() > 512 {
            return Err("Keep the format under 512 characters.".into());
        }
        if pattern
            .chars()
            .any(|c| c < ' ' || "<>:\"/\\|?*".contains(c))
        {
            return Err("The format contains a character Windows cannot use in a filename.".into());
        }
        let mut parts = Vec::new();
        let mut remaining = pattern;
        let mut has_name = false;
        while !remaining.is_empty() {
            if let Some(tail) = remaining.strip_prefix('[') {
                let end = tail.find(']').ok_or("Close each token with ].")?;
                match &tail[..end] {
                    "Movie Name" => {
                        parts.push(MoviePart::Name);
                        has_name = true;
                    }
                    "Movie" => parts.push(MoviePart::Text("Movie".into())),
                    "Space" => parts.push(MoviePart::Text(" ".into())),
                    token => return Err(format!("Unknown movie token [{token}].")),
                }
                remaining = &tail[end + 1..];
            } else {
                let end = remaining.find('[').unwrap_or(remaining.len());
                let text = &remaining[..end];
                if text.contains(']') {
                    return Err("Open each token with [.".into());
                }
                parts.push(MoviePart::Text(text.into()));
                remaining = &remaining[end..];
            }
        }
        if !has_name {
            return Err("Include [Movie Name] so each movie keeps its title.".into());
        }
        Ok(Self { parts })
    }
    pub fn render(&self, name: &str) -> String {
        let mut output = String::new();
        for part in &self.parts {
            match part {
                MoviePart::Name => output.push_str(name),
                MoviePart::Text(text) => output.push_str(text),
            }
        }
        output.trim().to_owned()
    }
}

/// Preserve filenames from saved formats created before explicit number tokens.
/// Old tokens are accepted here only for migration, never by the format editor.
pub fn migrate_legacy(pattern: &str) -> String {
    if ["[Num]", "[FixNum]", "[SeaNum]", "[EpsNum]"]
        .iter()
        .any(|token| pattern.contains(token))
    {
        return pattern.to_owned();
    }
    let mut output = String::new();
    let mut remaining = pattern;
    while let Some(start) = remaining.find('[') {
        output.push_str(&remaining[..start]);
        let tail = &remaining[start + 1..];
        let Some(end) = tail.find(']') else {
            output.push_str(&remaining[start..]);
            return output;
        };
        let token = &tail[..end];
        remaining = &tail[end + 1..];
        match token {
            "S" | "Season" | "E" | "Episode" => {
                let season = token == "S" || token == "Season";
                let fix = if season { "[FixSea]" } else { "[FixEps]" };
                output.push_str(&format!("[{token}]"));
                if let Some(rest) = remaining.strip_prefix(fix) {
                    output.push_str("[FixNum]");
                    remaining = rest;
                } else {
                    output.push_str("[Num]");
                }
                output.push_str(if season { "[SeaNum]" } else { "[EpsNum]" });
            }
            "FixSea" => output.push_str("[FixNum][SeaNum]"),
            "FixEps" => output.push_str("[FixNum][EpsNum]"),
            _ => output.push_str(&format!("[{token}]")),
        }
    }
    output.push_str(remaining);
    output
}

#[derive(Debug)]
enum Part {
    Text(String),
    Season(&'static str, bool),
    Episode(&'static str, bool),
}
#[derive(Debug)]
pub struct Format {
    parts: Vec<Part>,
    matcher: regex::Regex,
    season_groups: Vec<usize>,
    episode_groups: Vec<usize>,
}
impl Format {
    pub fn parse(pattern: &str) -> Result<Self, String> {
        if pattern.len() > 512 {
            return Err("Keep the format under 512 characters.".into());
        }
        if pattern
            .chars()
            .any(|c| c < ' ' || "<>:\"/\\|?*".contains(c))
        {
            return Err("The format contains a character Windows cannot use in a filename.".into());
        }
        let mut parts = Vec::new();
        let mut remaining = pattern;
        let mut padded = None;
        while !remaining.is_empty() {
            if let Some(tail) = remaining.strip_prefix('[') {
                let end = tail.find(']').ok_or("Close each token with ].")?;
                let token = &tail[..end];
                match token {
                    "S" | "Season" | "E" | "Episode" => parts.push(Part::Text(token.into())),
                    "Space" => parts.push(Part::Text(" ".into())),
                    "Num" => padded = Some(true),
                    "FixNum" => padded = Some(false),
                    "SeaNum" | "EpsNum" => {
                        let padded =
                            padded.ok_or("Choose [Num] or [FixNum] before a number token.")?;
                        parts.push(if token == "SeaNum" {
                            Part::Season("", padded)
                        } else {
                            Part::Episode("", padded)
                        });
                    }
                    _ => return Err(format!("Unknown token [{token}].")),
                }
                remaining = &tail[end + 1..];
            } else {
                let end = remaining.find('[').unwrap_or(remaining.len());
                let text = &remaining[..end];
                if text.contains(']') {
                    return Err("Open each token with [.".into());
                }
                parts.push(Part::Text(text.into()));
                remaining = &remaining[end..];
            }
        }
        if !parts.iter().any(|p| matches!(p, Part::Episode(..))) {
            return Err("Add [EpsNum] for the episode number.".into());
        }
        let mut preceding_number = false;
        for part in &parts {
            match part {
                Part::Text(text) => {
                    if text.chars().any(|c| !c.is_ascii_digit()) {
                        preceding_number = false;
                    }
                }
                Part::Season(label, _) | Part::Episode(label, _) => {
                    if label.is_empty() && preceding_number {
                        return Err(
                            "Separate season and episode numbers with text or [Space].".into()
                        );
                    }
                    preceding_number = true;
                }
            }
        }
        let mut expression = String::new();
        let mut season_groups = Vec::new();
        let mut episode_groups = Vec::new();
        let mut group = 0;
        for part in &parts {
            match part {
                Part::Text(text) => expression.push_str(&regex::escape(text)),
                Part::Season(label, _) | Part::Episode(label, _) => {
                    expression.push_str(&regex::escape(label));
                    expression.push_str("([0-9]{1,10})");
                    group += 1;
                    if matches!(part, Part::Season(..)) {
                        season_groups.push(group);
                    } else {
                        episode_groups.push(group);
                    }
                }
            }
        }
        let matcher = regex::Regex::new(&format!("(?:^| ){}$", expression.trim()))
            .map_err(|_| "This format is too complex.".to_owned())?;
        Ok(Self {
            parts,
            matcher,
            season_groups,
            episode_groups,
        })
    }
    /// Recognize previously renamed files so refresh keeps their season intact.
    pub fn detect(&self, name: &str) -> Option<(Option<u32>, u32)> {
        let captures = self.matcher.captures(name)?;
        let read = |groups: &[usize]| -> Option<u32> {
            let mut value = None;
            for group in groups {
                let n = captures.get(*group)?.as_str().parse::<u32>().ok()?;
                if value.is_some_and(|previous| previous != n) {
                    return None;
                }
                value = Some(n);
            }
            value
        };
        let season = if self.season_groups.is_empty() {
            None
        } else {
            Some(read(&self.season_groups)?)
        };
        Some((season, read(&self.episode_groups)?))
    }
    pub fn render(&self, season: u32, episode: u32) -> String {
        use std::fmt::Write;
        let mut output = String::new();
        for part in &self.parts {
            match part {
                Part::Text(text) => output.push_str(text),
                Part::Season(label, padded) | Part::Episode(label, padded) => {
                    let value = if matches!(part, Part::Season(..)) {
                        season
                    } else {
                        episode
                    };
                    output.push_str(label);
                    if *padded {
                        let _ = write!(output, "{value:02}");
                    } else {
                        let _ = write!(output, "{value}");
                    }
                }
            }
        }
        output.trim().to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn saved_legacy_formats_keep_their_output() {
        for (old, output) in [
            ("[S][E]", "S01E09"),
            ("[S][FixSea][Space][E][FixEps]", "S1 E9"),
            ("[FixEps]", "9"),
        ] {
            let converted = migrate_legacy(old);
            assert!(!converted.contains("[FixSea]") && !converted.contains("[FixEps]"));
            assert_eq!(Format::parse(&converted).unwrap().render(1, 9), output);
        }
        assert_eq!(migrate_legacy(DEFAULT_FORMAT), DEFAULT_FORMAT);
    }
    #[test]
    fn requested_formats_and_literal_text() {
        for (pattern, expected) in [
            ("[Num][S][SeaNum][E][EpsNum]", "S01E09"),
            ("[FixNum][S][SeaNum][Space][E][EpsNum]", "S1 E9"),
            (
                "[Num][Season][Space][SeaNum][Space][Episode][Space][EpsNum]",
                "Season 01 Episode 09",
            ),
            ("[S][FixNum][SeaNum][Space][E][Num][EpsNum]", "S1 E09"),
            ("[Num][E][EpsNum]", "E09"),
            ("[FixNum][EpsNum]", "9"),
            (
                "[FixNum]Part [SeaNum] - Episode [EpsNum]",
                "Part 1 - Episode 9",
            ),
            (
                "[Num][Season][Space][Episode][Space][EpsNum]",
                "Season Episode 09",
            ),
        ] {
            let format = Format::parse(pattern).unwrap();
            assert_eq!(format.render(1, 9), expected);
            assert_eq!(format.detect(&format!("Show {expected}")).unwrap().1, 9);
        }
        assert_eq!(
            Format::parse(DEFAULT_FORMAT).unwrap().render(12, 123),
            "S12E123"
        );
    }
    #[test]
    fn invalid_formats_fail_before_renaming() {
        for pattern in [
            "",
            "[S]",
            "[Unknown]",
            "[E",
            "E]",
            "[E]/file",
            "[E]:bad",
            "[E]\n",
            "[Num][SeaNum][EpsNum]",
            "[EpsNum]",
            "[S][SeaNum][E][EpsNum]",
            "[Num][EpsNum][FixSea]",
            "[Num][EpsNum][FixEps]",
        ] {
            assert!(Format::parse(pattern).is_err(), "{pattern}");
        }
    }
}

#[cfg(test)]
mod movie_tests {
    use super::*;
    #[test]
    fn movie_tokens_and_windows_filename_validation() {
        assert_eq!(
            MovieFormat::parse(DEFAULT_MOVIE_FORMAT)
                .unwrap()
                .render("Example"),
            "Example Movie"
        );
        assert_eq!(
            MovieFormat::parse("[Movie Name]")
                .unwrap()
                .render("Example"),
            "Example"
        );
        assert_eq!(
            MovieFormat::parse("[Movie][Space]-[Space][Movie Name]")
                .unwrap()
                .render("Example"),
            "Movie - Example"
        );
        for invalid in [
            "[Movie]",
            "[Movie Name",
            "[Movie Name]/[Movie]",
            "[EpsNum]",
            "[Movie Name]?",
        ] {
            assert!(MovieFormat::parse(invalid).is_err(), "{invalid}");
        }
    }
}
