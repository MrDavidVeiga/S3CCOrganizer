use serde::Serialize;
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceCfgRule {
    pub priority: i32,
    pub pattern: String,
    pub source_line: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceCfgInfo {
    pub path: String,
    pub rules: Vec<ResourceCfgRule>,
    pub warnings: Vec<String>,
    pub precedence_reliable: bool,
}

#[derive(Debug, Clone)]
pub struct PackagePriority {
    pub priority: i32,
    pub rule: String,
}

fn strip_comment(line: &str) -> &str {
    let hash = line.find('#');
    let semicolon = line.find(';');
    match (hash, semicolon) {
        (Some(a), Some(b)) => &line[..a.min(b)],
        (Some(a), None) => &line[..a],
        (None, Some(b)) => &line[..b],
        (None, None) => line,
    }
}

fn normalized_path_text(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn wildcard_match_component(pattern: &str, value: &str) -> bool {
    let pattern = pattern.as_bytes();
    let value = value.as_bytes();
    let mut dp = vec![vec![false; value.len() + 1]; pattern.len() + 1];
    dp[0][0] = true;

    for i in 1..=pattern.len() {
        if pattern[i - 1] == b'*' {
            dp[i][0] = dp[i - 1][0];
        }
    }

    for i in 1..=pattern.len() {
        for j in 1..=value.len() {
            dp[i][j] = if pattern[i - 1] == b'*' {
                dp[i - 1][j] || dp[i][j - 1]
            } else {
                pattern[i - 1].eq_ignore_ascii_case(&value[j - 1]) && dp[i - 1][j - 1]
            };
        }
    }

    dp[pattern.len()][value.len()]
}

fn wildcard_path_match(pattern: &str, value: &str) -> bool {
    let pattern_parts = pattern
        .replace('\\', "/")
        .split('/')
        .filter(|part| !part.is_empty())
        .map(|part| part.to_string())
        .collect::<Vec<_>>();
    let value_parts = value
        .replace('\\', "/")
        .split('/')
        .filter(|part| !part.is_empty())
        .map(|part| part.to_string())
        .collect::<Vec<_>>();

    if pattern_parts.len() != value_parts.len() {
        return false;
    }

    pattern_parts
        .iter()
        .zip(value_parts.iter())
        .all(|(pattern, value)| wildcard_match_component(pattern, value))
}

pub fn find_resource_cfg(selected_root: &Path) -> Option<PathBuf> {
    let direct = selected_root.join("Resource.cfg");
    if direct.is_file() {
        return Some(direct);
    }

    let parent = selected_root.parent()?.join("Resource.cfg");
    if parent.is_file() {
        return Some(parent);
    }

    None
}

pub fn parse_resource_cfg(path: &Path) -> Result<ResourceCfgInfo, String> {
    let text = fs::read_to_string(path)
        .map_err(|error| format!("Could not read {}: {error}", path.display()))?;

    let mut priority = 0i32;
    let mut rules = Vec::new();
    let mut warnings = Vec::new();
    let mut precedence_reliable = true;

    for (index, raw) in text.lines().enumerate() {
        let line_number = index + 1;
        let line = strip_comment(raw).trim();
        if line.is_empty() {
            continue;
        }

        let mut parts = line.split_whitespace();
        let Some(command) = parts.next() else {
            continue;
        };

        if matches!(
            command.to_ascii_lowercase().as_str(),
            "scan" | "select" | "end" | "stopscan"
        ) {
            precedence_reliable = false;
            warnings.push(format!(
                "Line {line_number}: '{command}' affects advanced Resource.cfg traversal/conditions and is not evaluated for precedence."
            ));
            continue;
        }

        if command.eq_ignore_ascii_case("DirectoryFiles") {
            warnings.push(format!(
                "Line {line_number}: DirectoryFiles is recognized as non-package loading metadata and is ignored by the package precedence analyzer."
            ));
            continue;
        }

        if command.eq_ignore_ascii_case("Priority") {
            let Some(value) = parts.next() else {
                warnings.push(format!("Line {line_number}: Priority has no value."));
                continue;
            };
            match value.parse::<i32>() {
                Ok(value) => priority = value,
                Err(_) => warnings.push(format!(
                    "Line {line_number}: invalid Priority value '{value}'."
                )),
            }
            continue;
        }

        if command.eq_ignore_ascii_case("PackedFile") {
            let Some(pattern) = parts.next() else {
                warnings.push(format!("Line {line_number}: PackedFile has no path."));
                continue;
            };
            rules.push(ResourceCfgRule {
                priority,
                pattern: pattern.replace('\\', "/"),
                source_line: line_number,
            });
        }
    }

    Ok(ResourceCfgInfo {
        path: path.to_string_lossy().to_string(),
        rules,
        warnings,
        precedence_reliable,
    })
}

pub fn package_priority(
    cfg: &ResourceCfgInfo,
    cfg_directory: &Path,
    package_path: &Path,
) -> Option<PackagePriority> {
    let relative = package_path.strip_prefix(cfg_directory).ok()?;
    let relative = normalized_path_text(relative);

    cfg.rules
        .iter()
        .filter(|rule| wildcard_path_match(&rule.pattern, &relative))
        .max_by_key(|rule| rule.priority)
        .map(|rule| PackagePriority {
            priority: rule.priority,
            rule: rule.pattern.clone(),
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wildcard_matches_one_directory_level_per_star() {
        assert!(wildcard_path_match(
            "Packages/*/*.package",
            "Packages/Hair/a.package"
        ));
        assert!(!wildcard_path_match(
            "Packages/*/*.package",
            "Packages/Hair/Female/a.package"
        ));
        assert!(wildcard_path_match(
            "Packages/*/*/*.package",
            "Packages/Hair/Female/a.package"
        ));
    }


    #[test]
    fn advanced_control_flow_marks_precedence_as_unreliable() {
        let dir = std::env::temp_dir();
        let path = dir.join("s3cc-organizer-resource-test.cfg");
        std::fs::write(
            &path,
            "Priority 500\nPackedFile Packages/*.package\nScan Overrides\n",
        )
        .unwrap();

        let parsed = parse_resource_cfg(&path).unwrap();
        let _ = std::fs::remove_file(&path);

        assert!(!parsed.precedence_reliable);
        assert!(parsed.warnings.iter().any(|warning| warning.contains("Scan")));
    }

    #[test]
    fn matching_is_case_insensitive() {
        assert!(wildcard_path_match(
            "Packages/*.package",
            "packages/Test.PACKAGE"
        ));
    }
}
