use std::error::Error;

/// Lowercase, replace non-alphanumeric with hyphens, collapse consecutive
/// hyphens, strip leading/trailing hyphens, truncate to 50 chars (at hyphen
/// boundary if possible).
pub fn slugify(name: &str) -> String {
    let lower = name.to_lowercase();
    let mut slug: String = lower
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect();
    // Collapse consecutive hyphens
    while slug.contains("--") {
        slug = slug.replace("--", "-");
    }
    // Strip leading/trailing hyphens
    let slug = slug.trim_matches('-').to_string();
    // Truncate to 50 chars at hyphen boundary if possible
    if slug.len() <= 50 {
        slug
    } else {
        let truncated = &slug[..50];
        if let Some(pos) = truncated.rfind('-') {
            truncated[..pos].to_string()
        } else {
            truncated.to_string()
        }
    }
}

/// Format: `{prefix}/sc-{id}-{slug}` where prefix defaults to `story_type`.
pub fn branch_name(story_type: &str, id: i64, name: &str, prefix_override: Option<&str>) -> String {
    let prefix = prefix_override.unwrap_or(story_type);
    let slug = slugify(name);
    format!("{prefix}/sc-{id}-{slug}")
}

/// Scan for `sc-{digits}` pattern anywhere in the branch name.
pub fn extract_story_id_from_branch(branch: &str) -> Option<i64> {
    let mut rest = branch;
    while let Some(pos) = rest.find("sc-") {
        let after_sc = &rest[pos + 3..];
        let digits: String = after_sc
            .chars()
            .take_while(|c| c.is_ascii_digit())
            .collect();
        if !digits.is_empty() {
            return digits.parse().ok();
        }
        rest = after_sc;
    }
    None
}

/// Abstraction over git operations for testability.
pub trait GitRunner {
    fn current_branch(&self) -> Result<String, Box<dyn Error>>;
    fn checkout_new_branch(&self, branch: &str) -> Result<(), Box<dyn Error>>;
    fn commit(&self, args: &[&str]) -> Result<String, Box<dyn Error>>;
}

/// Production implementation using `std::process::Command`.
pub struct RealGitRunner;

impl GitRunner for RealGitRunner {
    fn current_branch(&self) -> Result<String, Box<dyn Error>> {
        let output = std::process::Command::new("git")
            .args(["rev-parse", "--abbrev-ref", "HEAD"])
            .output()
            .map_err(|e| format!("Failed to run git: {e}"))?;
        if !output.status.success() {
            return Err(format!(
                "git rev-parse failed: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            )
            .into());
        }
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    fn checkout_new_branch(&self, branch: &str) -> Result<(), Box<dyn Error>> {
        let output = std::process::Command::new("git")
            .args(["checkout", "-b", branch])
            .output()
            .map_err(|e| format!("Failed to run git: {e}"))?;
        if !output.status.success() {
            return Err(format!(
                "git checkout failed: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            )
            .into());
        }
        Ok(())
    }

    fn commit(&self, args: &[&str]) -> Result<String, Box<dyn Error>> {
        let output = std::process::Command::new("git")
            .arg("commit")
            .args(args)
            .output()
            .map_err(|e| format!("Failed to run git: {e}"))?;
        if !output.status.success() {
            return Err(format!(
                "git commit failed: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            )
            .into());
        }
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slugify_basic() {
        assert_eq!(slugify("Fix Login Bug"), "fix-login-bug");
    }

    #[test]
    fn slugify_special_chars() {
        assert_eq!(slugify("Add API (v2) support!"), "add-api-v2-support");
    }

    #[test]
    fn slugify_leading_trailing_hyphens() {
        assert_eq!(slugify("--hello--world--"), "hello-world");
    }

    #[test]
    fn slugify_truncates_long_names() {
        let name = "this is a very long story name that will definitely exceed the fifty character limit set";
        let slug = slugify(name);
        assert!(slug.len() <= 50);
        assert!(!slug.ends_with('-'));
    }

    #[test]
    fn slugify_truncates_at_hyphen_boundary() {
        // 51 chars as slug: "abcdefghij-abcdefghij-abcdefghij-abcdefghij-abcdefg"
        let name = "abcdefghij abcdefghij abcdefghij abcdefghij abcdefghij";
        let slug = slugify(name);
        assert!(slug.len() <= 50);
        // Should have truncated at a hyphen boundary
        assert!(!slug.ends_with('-'));
    }

    #[test]
    fn branch_name_default_prefix() {
        assert_eq!(
            branch_name("feature", 123, "Fix Login Bug", None),
            "feature/sc-123-fix-login-bug"
        );
    }

    #[test]
    fn branch_name_custom_prefix() {
        assert_eq!(
            branch_name("feature", 123, "Fix Login Bug", Some("hotfix")),
            "hotfix/sc-123-fix-login-bug"
        );
    }

    #[test]
    fn branch_name_bug_type() {
        assert_eq!(
            branch_name("bug", 456, "Crash on load", None),
            "bug/sc-456-crash-on-load"
        );
    }

    #[test]
    fn extract_id_standard() {
        assert_eq!(
            extract_story_id_from_branch("feature/sc-123-fix-bug"),
            Some(123)
        );
    }

    #[test]
    fn extract_id_no_prefix() {
        assert_eq!(extract_story_id_from_branch("sc-456-some-name"), Some(456));
    }

    #[test]
    fn extract_id_nested() {
        assert_eq!(
            extract_story_id_from_branch("user/feature/sc-789-thing"),
            Some(789)
        );
    }

    #[test]
    fn extract_id_no_match() {
        assert_eq!(extract_story_id_from_branch("main"), None);
        assert_eq!(extract_story_id_from_branch("feature/no-story"), None);
    }

    #[test]
    fn extract_id_sc_without_digits() {
        assert_eq!(extract_story_id_from_branch("feature/sc-abc"), None);
    }
}
