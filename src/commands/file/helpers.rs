use std::error::Error;
use std::path::{Path, PathBuf};

/// A local file that has been validated and is ready to upload.
#[derive(Debug, Clone)]
pub struct PreparedFile {
    pub path: PathBuf,
    pub filename: String,
    pub content_type: String,
    pub size: u64,
}

/// Validate every path before anything is uploaded.
///
/// Each path must exist, be a regular file, and be non-empty. The MIME type is
/// guessed from the extension, falling back to `application/octet-stream`.
pub fn prepare_files(paths: &[PathBuf]) -> Result<Vec<PreparedFile>, Box<dyn Error>> {
    if paths.is_empty() {
        return Err("At least one file path is required".into());
    }
    paths.iter().map(|p| prepare_file(p)).collect()
}

fn prepare_file(path: &Path) -> Result<PreparedFile, Box<dyn Error>> {
    let meta = std::fs::metadata(path)
        .map_err(|e| format!("Cannot read file '{}': {e}", path.display()))?;
    if !meta.is_file() {
        return Err(format!("'{}' is not a regular file", path.display()).into());
    }
    if meta.len() == 0 {
        return Err(format!("'{}' is empty", path.display()).into());
    }
    let filename = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .ok_or_else(|| format!("'{}' has no file name", path.display()))?;
    let content_type = mime_guess::from_path(path)
        .first_or_octet_stream()
        .essence_str()
        .to_string();
    Ok(PreparedFile {
        path: path.to_path_buf(),
        filename,
        content_type,
        size: meta.len(),
    })
}

/// Render a ready-to-paste markdown snippet for an uploaded file:
/// `![name](url)` for images, `[name](url)` for everything else.
pub fn markdown_for(name: &str, url: &str, content_type: &str) -> String {
    let label = name.replace('[', "\\[").replace(']', "\\]");
    let target = if url.contains([' ', '(', ')']) {
        format!("<{url}>")
    } else {
        url.to_string()
    };
    if content_type.starts_with("image/") {
        format!("![{label}]({target})")
    } else {
        format!("[{label}]({target})")
    }
}

/// Markdown snippet for a raw `UploadedFile` JSON object, if it has a URL.
pub fn markdown_for_value(file: &serde_json::Value) -> Option<String> {
    let url = file.get("url").and_then(|v| v.as_str())?;
    let name = file
        .get("name")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .or_else(|| file.get("filename").and_then(|v| v.as_str()))
        .unwrap_or("file");
    let content_type = file
        .get("content_type")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    Some(markdown_for(name, url, content_type))
}

/// Human-readable byte size, e.g. `34.2 KB`.
pub fn human_size(bytes: u64) -> String {
    const UNITS: [&str; 4] = ["B", "KB", "MB", "GB"];
    let mut size = bytes as f64;
    let mut unit = 0;
    while size >= 1024.0 && unit < UNITS.len() - 1 {
        size /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{bytes} B")
    } else {
        format!("{size:.1} {}", UNITS[unit])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn markdown_image_vs_link() {
        assert_eq!(
            markdown_for("a.png", "https://x/a.png", "image/png"),
            "![a.png](https://x/a.png)"
        );
        assert_eq!(
            markdown_for("spec.pdf", "https://x/spec.pdf", "application/pdf"),
            "[spec.pdf](https://x/spec.pdf)"
        );
    }

    #[test]
    fn markdown_escapes_brackets_and_wraps_odd_urls() {
        assert_eq!(
            markdown_for("a [1].png", "https://x/a (1).png", "image/png"),
            "![a \\[1\\].png](<https://x/a (1).png>)"
        );
    }

    #[test]
    fn human_size_units() {
        assert_eq!(human_size(512), "512 B");
        assert_eq!(human_size(2048), "2.0 KB");
        assert_eq!(human_size(5 * 1024 * 1024), "5.0 MB");
    }

    #[test]
    fn prepare_rejects_missing_and_empty() {
        let dir = tempfile::tempdir().unwrap();
        let empty = dir.path().join("empty.txt");
        std::fs::write(&empty, b"").unwrap();
        assert!(prepare_files(&[dir.path().join("nope.png")]).is_err());
        assert!(prepare_files(&[empty]).is_err());
        assert!(prepare_files(&[dir.path().to_path_buf()]).is_err());
    }

    #[test]
    fn prepare_guesses_mime() {
        let dir = tempfile::tempdir().unwrap();
        let png = dir.path().join("shot.png");
        let bin = dir.path().join("blob.unknownext");
        std::fs::write(&png, b"x").unwrap();
        std::fs::write(&bin, b"x").unwrap();
        let files = prepare_files(&[png, bin]).unwrap();
        assert_eq!(files[0].content_type, "image/png");
        assert_eq!(files[0].filename, "shot.png");
        assert_eq!(files[1].content_type, "application/octet-stream");
    }
}
