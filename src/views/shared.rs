use comrak::plugins::syntect::SyntectAdapter;
use comrak::{ComrakOptions, ComrakPlugins, markdown_to_html_with_plugins};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(not(target_arch = "wasm32"))]
use std::{fs, path::PathBuf};

pub const SAVED_DOCS_DIR: &str = "cache/docs";

static MARKDOWN_OPTIONS: Lazy<ComrakOptions> = Lazy::new(|| {
    let mut options = ComrakOptions::default();
    options.extension.table = true;
    options.extension.footnotes = true;
    options.extension.strikethrough = true;
    options.extension.tasklist = true;
    options.render.unsafe_ = true;
    options
});

#[derive(Clone, PartialEq)]
pub struct SavedDoc {
    pub id: String,
    pub title: String,
    pub content: String,
    pub file_path: Option<String>,
    pub created_at: u64,
    pub tags: Vec<String>,
}

const STOPWORDS: &[&str] = &[
    "the", "and", "that", "with", "have", "this", "from", "there", "would", "could", "should",
    "about", "into", "while", "where", "which", "their", "them", "they", "been", "after", "before",
    "because", "given", "using", "based", "over", "under", "through", "among",
];

pub fn generate_tags(text: &str) -> Vec<String> {
    let mut frequencies: HashMap<String, usize> = HashMap::new();

    for word in text.split(|c: char| c.is_whitespace()) {
        let cleaned = word
            .trim_matches(|c: char| !c.is_alphanumeric())
            .to_lowercase();
        if cleaned.len() < 4 || STOPWORDS.contains(&cleaned.as_str()) {
            continue;
        }
        *frequencies.entry(cleaned).or_insert(0) += 1;
    }

    let mut items: Vec<(String, usize)> = frequencies.into_iter().collect();
    items.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));

    items
        .into_iter()
        .take(4)
        .map(|(word, _)| capitalize_tag(&word))
        .collect()
}

fn capitalize_tag(word: &str) -> String {
    let mut chars = word.chars();
    match chars.next() {
        Some(first) => format!("{}{}", first.to_uppercase(), chars.as_str()),
        None => String::new(),
    }
}

pub fn markdown_to_html(md: &str) -> String {
    let adapter = SyntectAdapter::new(Some("base16-ocean.dark"));
    let mut plugins = ComrakPlugins::default();
    plugins.render.codefence_syntax_highlighter = Some(&adapter);
    markdown_to_html_with_plugins(md, &MARKDOWN_OPTIONS, &plugins)
}

pub fn initial_saved_docs() -> Vec<SavedDoc> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        load_docs_from_disk()
    }
    #[cfg(target_arch = "wasm32")]
    {
        Vec::new()
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn persist_markdown_doc(content: &str, tags_override: Option<&[String]>) -> Option<SavedDoc> {
    if content.trim().is_empty() {
        return None;
    }

    if let Err(err) = fs::create_dir_all(SAVED_DOCS_DIR) {
        eprintln!("failed to create docs directory: {}", err);
        return None;
    }

    let timestamp = current_timestamp();
    let title = extract_title(content, "Untitled");
    let slug = slugify_for_filename(&title);
    let filename = if slug.is_empty() {
        format!("note-{}.md", timestamp)
    } else {
        format!("{}-{}.md", slug, timestamp)
    };
    let path = PathBuf::from(SAVED_DOCS_DIR).join(filename);
    if let Err(err) = fs::write(&path, content) {
        eprintln!("failed to write saved doc: {}", err);
        return None;
    }
    let path_str = path.to_string_lossy().into_owned();
    let tags = match tags_override {
        Some(tags) if !tags.is_empty() => tags.to_vec(),
        _ => generate_tags(content),
    };
    Some(SavedDoc {
        id: path_str.clone(),
        title,
        content: content.to_string(),
        file_path: Some(path_str),
        created_at: timestamp,
        tags,
    })
}

#[cfg(target_arch = "wasm32")]
pub fn persist_markdown_doc(content: &str, tags_override: Option<&[String]>) -> Option<SavedDoc> {
    if content.trim().is_empty() {
        return None;
    }

    let timestamp = current_timestamp();
    let title = extract_title(content, "Untitled");
    let tags = match tags_override {
        Some(tags) if !tags.is_empty() => tags.to_vec(),
        _ => generate_tags(content),
    };
    Some(SavedDoc {
        id: format!("mem-{}", timestamp),
        title,
        content: content.to_string(),
        file_path: None,
        created_at: timestamp,
        tags,
    })
}

pub fn display_file_name(path: &str) -> String {
    Path::new(path)
        .file_name()
        .and_then(|stem| stem.to_str())
        .unwrap_or(path)
        .to_string()
}

#[cfg(not(target_arch = "wasm32"))]
fn load_docs_from_disk() -> Vec<SavedDoc> {
    let dir = PathBuf::from(SAVED_DOCS_DIR);
    if !dir.exists() {
        return Vec::new();
    }

    let mut docs_with_time: Vec<(u64, SavedDoc)> = Vec::new();
    if let Ok(entries) = fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) != Some("md") {
                continue;
            }
            if let Ok(content) = fs::read_to_string(&path) {
                let fallback = path
                    .file_stem()
                    .and_then(|stem| stem.to_str())
                    .unwrap_or("Untitled");
                let title = extract_title(&content, fallback);
                let tags = generate_tags(&content);
                let timestamp = entry
                    .metadata()
                    .ok()
                    .and_then(|meta| meta.modified().ok())
                    .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
                    .map(|dur| dur.as_secs())
                    .unwrap_or(0);
                let path_str = path.to_string_lossy().into_owned();
                let doc = SavedDoc {
                    id: path_str.clone(),
                    title,
                    content,
                    file_path: Some(path_str),
                    created_at: timestamp,
                    tags,
                };
                docs_with_time.push((doc.created_at, doc));
            }
        }
    }

    docs_with_time.sort_by(|a, b| b.0.cmp(&a.0));
    docs_with_time.into_iter().map(|(_, doc)| doc).collect()
}

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn extract_title(content: &str, fallback: &str) -> String {
    let candidate = content
        .lines()
        .find_map(|line| {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.trim_start_matches('#').trim().to_string())
            }
        })
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| fallback.to_string());

    truncate_title(candidate)
}

fn truncate_title(mut title: String) -> String {
    if title.len() > 80 {
        title.truncate(80);
    }
    title
}

fn slugify_for_filename(input: &str) -> String {
    let mut slug = String::new();
    let mut last_dash = false;
    for ch in input.chars() {
        let lower = ch.to_ascii_lowercase();
        if lower.is_ascii_alphanumeric() {
            slug.push(lower);
            last_dash = false;
        } else if (lower.is_ascii_whitespace() || lower == '-') && !last_dash && !slug.is_empty() {
            slug.push('-');
            last_dash = true;
        }
        if slug.len() >= 40 {
            break;
        }
    }
    slug.trim_matches('-').to_string()
}
