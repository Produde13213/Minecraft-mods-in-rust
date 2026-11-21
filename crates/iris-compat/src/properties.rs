//! Parser for the common key/value subset of shader-pack properties files.
//! Unknown keys are preserved so higher-level Iris compatibility code can interpret them.

use std::collections::BTreeMap;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Properties {
    values: BTreeMap<String, String>,
    pub diagnostics: Vec<PropertyDiagnostic>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PropertyDiagnostic {
    pub line: usize,
    pub message: String,
}

impl Properties {
    pub fn parse(input: &str) -> Self {
        let mut result = Self::default();
        for (index, raw) in input.lines().enumerate() {
            let line_no = index + 1;
            let line = raw.trim();
            if line.is_empty() || line.starts_with('#') || line.starts_with('!') { continue; }
            let split = line.find(|c: char| c == '=' || c == ':');
            let Some(pos) = split else {
                result.diagnostics.push(PropertyDiagnostic {
                    line: line_no,
                    message: "expected key=value or key:value".into(),
                });
                continue;
            };
            let key = line[..pos].trim();
            let value = line[pos + 1..].trim();
            if key.is_empty() {
                result.diagnostics.push(PropertyDiagnostic { line: line_no, message: "property key is empty".into() });
                continue;
            }
            result.values.insert(key.to_owned(), unescape(value));
        }
        result
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.values.get(key).map(String::as_str)
    }

    pub fn contains_key(&self, key: &str) -> bool { self.values.contains_key(key) }

    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        self.values.iter().map(|(k, v)| (k.as_str(), v.as_str()))
    }

    pub fn get_bool(&self, key: &str) -> Option<bool> {
        match self.get(key)?.trim().to_ascii_lowercase().as_str() {
            "true" | "yes" | "1" | "on" => Some(true),
            "false" | "no" | "0" | "off" => Some(false),
            _ => None,
        }
    }

    pub fn get_list(&self, key: &str) -> Option<Vec<&str>> {
        self.get(key).map(|value| value.split_whitespace().collect())
    }
}

fn unescape(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    let mut chars = value.chars();
    while let Some(ch) = chars.next() {
        if ch == '\\' {
            match chars.next() {
                Some('n') => out.push('\n'),
                Some('r') => out.push('\r'),
                Some('t') => out.push('\t'),
                Some(other) => out.push(other),
                None => out.push('\\'),
            }
        } else {
            out.push(ch);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_comments_delimiters_and_values() {
        let props = Properties::parse("# comment\n! another\nclouds = off\nprofile: high\ntext = hello\\nworld");
        assert_eq!(props.get_bool("clouds"), Some(false));
        assert_eq!(props.get("profile"), Some("high"));
        assert_eq!(props.get("text"), Some("hello\nworld"));
        assert!(props.diagnostics.is_empty());
    }

    #[test]
    fn last_duplicate_value_wins_and_lists_split_on_whitespace() {
        let props = Properties::parse("program = shadow\nprogram = shadowcomp\nfeatures = WATER SHADOWS");
        assert_eq!(props.get("program"), Some("shadowcomp"));
        assert_eq!(props.get_list("features"), Some(vec!["WATER", "SHADOWS"]));
    }

    #[test]
    fn malformed_lines_are_reported_without_panicking() {
        let props = Properties::parse("broken\n=missing-key\nok=yes");
        assert_eq!(props.diagnostics.len(), 2);
        assert_eq!(props.get("ok"), Some("yes"));
    }
}
