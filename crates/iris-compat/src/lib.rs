//! Clean-room shader-pack compatibility primitives.
//!
//! This crate starts with bounded, source-mapped GLSL include expansion. It is
//! intentionally independent of Iris source code and does not claim full Iris
//! compatibility. The host renderer supplies shader files through `ShaderSource`.

use std::collections::HashMap;
use std::fmt;

pub const MAX_INCLUDE_DEPTH: usize = 10;
pub const MAX_SOURCE_BYTES: usize = 8 * 1024 * 1024;

pub trait ShaderSource {
    fn read(&self, path: &str) -> Result<String, String>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceLocation {
    pub file: String,
    pub line: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShaderDiagnostic {
    pub location: SourceLocation,
    pub message: String,
}

impl fmt::Display for ShaderDiagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}: {}", self.location.file, self.location.line, self.message)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ExpandedShader {
    pub source: String,
    /// One entry per emitted line, including blank lines retained for includes.
    pub line_map: Vec<SourceLocation>,
    pub files: Vec<String>,
    pub diagnostics: Vec<ShaderDiagnostic>,
}

pub fn normalize_pack_path(path: &str) -> Option<String> {
    let replaced = path.replace('\\', "/");
    if replaced.starts_with('/') || replaced.contains(':') {
        return None;
    }
    let mut parts: Vec<&str> = Vec::new();
    for part in replaced.split('/') {
        match part {
            "" | "." => {}
            ".." => { if parts.pop().is_none() { return None; } }
            other => parts.push(other),
        }
    }
    if parts.is_empty() { None } else { Some(parts.join("/")) }
}

fn resolve_include(parent: &str, requested: &str) -> Option<String> {
    if requested.starts_with('/') {
        return normalize_pack_path(requested.trim_start_matches('/'));
    }
    let parent_dir = parent.rsplit_once('/').map(|(d, _)| d).unwrap_or("");
    let joined = if parent_dir.is_empty() {
        requested.to_owned()
    } else {
        format!("{parent_dir}/{requested}")
    };
    normalize_pack_path(&joined)
}

pub fn expand_includes<S: ShaderSource>(source: &S, entry: &str) -> ExpandedShader {
    let mut state = Expansion {
        source,
        output: ExpandedShader::default(),
        cache: HashMap::new(),
        active: Vec::new(),
        bytes_read: 0,
    };
    match normalize_pack_path(entry) {
        Some(path) => state.expand_file(&path, 0),
        None => state.error(entry, 1, "entry path escapes the shader pack"),
    }
    state.output
}

struct Expansion<'a, S> {
    source: &'a S,
    output: ExpandedShader,
    cache: HashMap<String, Result<String, String>>,
    active: Vec<String>,
    bytes_read: usize,
}

impl<S: ShaderSource> Expansion<'_, S> {
    fn error(&mut self, file: &str, line: usize, message: impl Into<String>) {
        self.output.diagnostics.push(ShaderDiagnostic {
            location: SourceLocation { file: file.to_owned(), line },
            message: message.into(),
        });
    }

    fn emit(&mut self, text: &str, file: &str, line: usize) {
        self.output.source.push_str(text);
        self.output.source.push('\n');
        self.output.line_map.push(SourceLocation { file: file.to_owned(), line });
    }

    fn expand_file(&mut self, path: &str, depth: usize) {
        if depth >= MAX_INCLUDE_DEPTH {
            self.error(path, 1, format!("include nesting exceeds {MAX_INCLUDE_DEPTH}"));
            return;
        }
        if self.active.iter().any(|p| p == path) {
            self.error(path, 1, "recursive include cycle detected");
            return;
        }
        let content = if let Some(cached) = self.cache.get(path) {
            cached.clone()
        } else {
            let loaded = self.source.read(path);
            if let Ok(text) = &loaded {
                self.bytes_read = self.bytes_read.saturating_add(text.len());
                if self.bytes_read > MAX_SOURCE_BYTES {
                    self.error(path, 1, "expanded source exceeds the 8 MiB input budget");
                    return;
                }
            }
            self.cache.insert(path.to_owned(), loaded.clone());
            loaded
        };
        let text = match content {
            Ok(text) => text,
            Err(message) => {
                self.error(path, 1, format!("cannot read include: {message}"));
                return;
            }
        };
        if !self.output.files.iter().any(|f| f == path) {
            self.output.files.push(path.to_owned());
        }
        self.active.push(path.to_owned());
        for (index, line) in text.lines().enumerate() {
            let number = index + 1;
            if let Some(requested) = parse_include(line) {
                match resolve_include(path, requested) {
                    Some(target) => {
                        // Keep the directive's line so downstream diagnostics remain aligned.
                        self.emit("", path, number);
                        self.expand_file(&target, depth + 1);
                    }
                    None => {
                        self.error(path, number, "include path escapes the shader pack");
                        self.emit("", path, number);
                    }
                }
            } else {
                self.emit(line, path, number);
            }
        }
        self.active.pop();
    }
}

fn parse_include(line: &str) -> Option<&str> {
    let trimmed = line.trim();
    let rest = trimmed.strip_prefix("#include")?.trim();
    if let Some(inner) = rest.strip_prefix('"') {
        let end = inner.find('"')?;
        if inner[end + 1..].trim().is_empty() { return Some(&inner[..end]); }
    }
    if let Some(inner) = rest.strip_prefix('<') {
        let end = inner.find('>')?;
        if inner[end + 1..].trim().is_empty() { return Some(&inner[..end]); }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    struct MemoryPack(BTreeMap<String, String>);
    impl ShaderSource for MemoryPack {
        fn read(&self, path: &str) -> Result<String, String> {
            self.0.get(path).cloned().ok_or_else(|| "not found".into())
        }
    }

    #[test]
    fn expands_relative_and_root_includes_with_source_map() {
        let pack = MemoryPack(BTreeMap::from([
            ("shaders/main.fsh".into(), "#version 330\n#include \"lib/math.glsl\"\nvoid main() {}".into()),
            ("shaders/lib/math.glsl".into(), "float twice(float x) { return x * 2.0; }".into()),
        ]));
        let result = expand_includes(&pack, "shaders/main.fsh");
        assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
        assert!(result.source.contains("float twice"));
        assert_eq!(result.files.len(), 2);
        assert_eq!(result.line_map[2].file, "shaders/lib/math.glsl");
    }

    #[test]
    fn rejects_path_traversal() {
        assert_eq!(normalize_pack_path("shaders/../../secret"), None);
        let pack = MemoryPack(BTreeMap::from([
            ("shaders/main.fsh".into(), "#include \"../../secret\"".into()),
        ]));
        let result = expand_includes(&pack, "shaders/main.fsh");
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].message.contains("escapes"));
    }

    #[test]
    fn reports_recursive_include_cycles() {
        let pack = MemoryPack(BTreeMap::from([
            ("shaders/a.glsl".into(), "#include \"b.glsl\"".into()),
            ("shaders/b.glsl".into(), "#include \"a.glsl\"".into()),
        ]));
        let result = expand_includes(&pack, "shaders/a.glsl");
        assert!(result.diagnostics.iter().any(|d| d.message.contains("cycle")));
    }
}
