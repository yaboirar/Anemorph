//! Compile the user's episode format once, then reuse it for each file.
pub const DEFAULT_FORMAT: &str = "[Series Name][Space][Num][S][SeaNum][E][EpsNum]";

pub const DEFAULT_MOVIE_FORMAT: &str = "[Movie Name] Movie";

#[derive(Debug)]
enum MoviePart {
    Text(String),
    Name,
    OriginalName,
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
                let end = tail.find(']').ok_or("Close each index with ].")?;
                match &tail[..end] {
                    "Movie Name" => {
                        parts.push(MoviePart::Name);
                        has_name = true;
                    }
                    "Movie" => parts.push(MoviePart::Text("Movie".into())),
                    "Original Name" => {
                        parts.push(MoviePart::OriginalName);
                        has_name = true;
                    }
                    "Num" | "FixNum" => {}
                    "Space" => parts.push(MoviePart::Text(" ".into())),
                    token => return Err(format!("Unknown movie index [{token}].")),
                }
                remaining = &tail[end + 1..];
            } else {
                let end = remaining.find('[').unwrap_or(remaining.len());
                let text = &remaining[..end];
                if text.contains(']') {
                    return Err("Open each index with [.".into());
                }
                parts.push(MoviePart::Text(text.into()));
                remaining = &remaining[end..];
            }
        }
        if !has_name {
            return Err(
                "Include [Movie Name] or [Original Name] so each movie keeps its title.".into(),
            );
        }
        Ok(Self { parts })
    }
    #[cfg(test)]
    pub fn render(&self, name: &str) -> String {
        self.render_name(name, name)
    }
    pub fn uses_movie_name(&self) -> bool {
        self.parts
            .iter()
            .any(|part| matches!(part, MoviePart::Name))
    }
    pub fn render_name(&self, name: &str, original: &str) -> String {
        let mut output = String::new();
        for part in &self.parts {
            match part {
                MoviePart::Name => output.push_str(name),
                MoviePart::OriginalName => output.push_str(original),
                MoviePart::Text(text) => output.push_str(text),
            }
        }
        if self
            .parts
            .iter()
            .any(|p| matches!(p, MoviePart::OriginalName))
        {
            output.trim().to_owned()
        } else {
            clean_spaces(&output)
        }
    }
}

/// Preserve filenames from saved formats created before explicit number tokens.
/// Old tokens are accepted here only for migration, never by the format editor.
pub fn migrate_legacy(pattern: &str) -> String {
    if [
        "[Num]",
        "[FixNum]",
        "[SeaNum]",
        "[EpsNum]",
        "[Series Name]",
        "[Episode Name]",
        "[Original Name]",
    ]
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
    SeriesName,
    EpisodeName,
    OriginalName,
}
#[derive(Debug)]
pub struct Format {
    parts: Vec<Part>,
    matcher: regex::Regex,
    season_groups: Vec<usize>,
    episode_groups: Vec<usize>,
    title_group: Option<usize>,
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
                let end = tail.find(']').ok_or("Close each index with ].")?;
                let token = &tail[..end];
                match token {
                    "S" | "Season" | "E" | "Episode" => parts.push(Part::Text(token.into())),
                    "Space" => parts.push(Part::Text(" ".into())),
                    "Series Name" => parts.push(Part::SeriesName),
                    "Episode Name" => parts.push(Part::EpisodeName),
                    "Original Name" => parts.push(Part::OriginalName),
                    "Num" => padded = Some(true),
                    "FixNum" => padded = Some(false),
                    "SeaNum" | "EpsNum" => {
                        let padded =
                            padded.ok_or("Choose [Num] or [FixNum] before a number index.")?;
                        parts.push(if token == "SeaNum" {
                            Part::Season("", padded)
                        } else {
                            Part::Episode("", padded)
                        });
                    }
                    _ => return Err(format!("Unknown episode index [{token}].")),
                }
                remaining = &tail[end + 1..];
            } else {
                let end = remaining.find('[').unwrap_or(remaining.len());
                let text = &remaining[..end];
                if text.contains(']') {
                    return Err("Open each index with [.".into());
                }
                parts.push(Part::Text(text.into()));
                remaining = &remaining[end..];
            }
        }
        if !parts.iter().any(|p| {
            matches!(
                p,
                Part::Episode(..) | Part::EpisodeName | Part::OriginalName
            )
        }) {
            return Err(
                "Add [EpsNum], [Episode Name], or [Original Name] to identify each episode.".into(),
            );
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
                Part::SeriesName | Part::EpisodeName | Part::OriginalName => {
                    preceding_number = false
                }
            }
        }
        let mut expression = String::new();
        let mut season_groups = Vec::new();
        let mut episode_groups = Vec::new();
        let mut group = 0;
        let mut title_group = None;
        for part in &parts {
            match part {
                Part::Text(text) if text.trim().is_empty() => expression.push_str(r"\s*"),
                Part::Text(text) => expression.push_str(&regex::escape(text)),
                Part::SeriesName | Part::OriginalName => expression.push_str("(?:.+?)"),
                Part::EpisodeName => {
                    expression.push_str("(.*?)");
                    group += 1;
                    title_group.get_or_insert(group);
                }
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
            title_group,
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
    #[cfg(test)]
    pub fn render(&self, season: u32, episode: u32) -> String {
        self.render_name("", "", "", season, episode)
    }
    pub fn uses_series_name(&self) -> bool {
        self.parts.iter().any(|p| matches!(p, Part::SeriesName))
    }
    pub fn uses_episode_number(&self) -> bool {
        !self.episode_groups.is_empty()
    }
    pub fn uses_episode_name(&self) -> bool {
        self.title_group.is_some()
    }
    pub fn episode_name<'a>(&self, name: &'a str) -> Option<&'a str> {
        self.matcher
            .captures(name)?
            .get(self.title_group?)
            .map(|m| m.as_str().trim())
            .filter(|name| !name.is_empty())
    }
    pub fn render_name(
        &self,
        series: &str,
        title: &str,
        original: &str,
        season: u32,
        episode: u32,
    ) -> String {
        use std::fmt::Write;
        let mut output = String::new();
        for part in &self.parts {
            match part {
                Part::Text(text) => output.push_str(text),
                Part::SeriesName => output.push_str(series),
                Part::EpisodeName => output.push_str(title),
                Part::OriginalName => output.push_str(original),
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
        if self.parts.iter().any(|p| matches!(p, Part::OriginalName)) {
            output.trim().to_owned()
        } else {
            clean_spaces(&output)
        }
    }
}

fn clean_spaces(output: &str) -> String {
    let mut clean = String::with_capacity(output.len());
    for word in output.split_whitespace() {
        if !clean.is_empty() {
            clean.push(' ');
        }
        clean.push_str(word);
    }
    clean
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn complete_names_title_capture_and_original_names() {
        let format =
            Format::parse("[Series Name][Space][Num][S][SeaNum][E][EpsNum][Space][Episode Name]")
                .unwrap();
        let result = format.render_name("Tamako Market", "She's the Cute Girl", "raw", 1, 9);
        assert_eq!(result, "Tamako Market S01E09 She's the Cute Girl");
        assert_eq!(format.detect(&result), Some((Some(1), 9)));
        assert_eq!(format.episode_name(&result), Some("She's the Cute Girl"));
        assert_eq!(
            format.render_name("Tamako Market", "", "raw", 1, 9),
            "Tamako Market S01E09"
        );
        assert_eq!(format.detect("Tamako Market S01E09"), Some((Some(1), 9)));
        assert_eq!(
            Format::parse("[Num][Episode][Space][EpsNum]")
                .unwrap()
                .render_name("Not added", "", "raw", 1, 9),
            "Episode 09"
        );
        assert_eq!(
            Format::parse("[Original Name]").unwrap().render_name(
                "",
                "",
                "Original  recording",
                1,
                9
            ),
            "Original  recording"
        );
        assert_eq!(
            MovieFormat::parse("[Original Name]")
                .unwrap()
                .render_name("clean", "Original  movie"),
            "Original  movie"
        );
    }
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
    fn episode_and_movie_indexes_are_strictly_separate() {
        for index in ["[Movie Name]", "[Movie]"] {
            assert!(Format::parse(&format!("{DEFAULT_FORMAT}{index}")).is_err());
        }
        for index in [
            "[S]",
            "[E]",
            "[Season]",
            "[Episode]",
            "[SeaNum]",
            "[EpsNum]",
            "[Series Name]",
            "[Episode Name]",
        ] {
            assert!(MovieFormat::parse(&format!("[Movie Name]{index}")).is_err());
        }
        assert!(Format::parse("[Num][EpsNum][Space]").is_ok());
        assert!(MovieFormat::parse("[Movie Name][Space][Movie]").is_ok());
    }
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
