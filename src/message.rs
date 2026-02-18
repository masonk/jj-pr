use lazy_regex::regex;

/// Parse a PR number from a commit message
/// Looks for patterns like:
/// - "Pull Request: #123"
/// - "Pull Request: https://github.com/owner/repo/pull/123"
pub fn parse_pr_number(message: &str) -> Option<u64> {
    // Match "Pull Request: #123" or "Pull Request: https://github.com/.../pull/123"
    let re = regex!(r"(?im)^Pull Request:\s*(?:#(\d+)|https?://[^\s]+/pull/(\d+))");

    if let Some(caps) = re.captures(message) {
        // Try the first capture group (#123), then the second (URL)
        if let Some(num) = caps.get(1).or_else(|| caps.get(2)) {
            return num.as_str().parse().ok();
        }
    }

    None
}

/// Add or update PR number in commit message
pub fn add_pr_number(message: &str, pr_number: u64) -> String {
    // Check if PR number already exists
    if parse_pr_number(message).is_some() {
        // Replace existing PR number
        let re = regex!(r"(?im)^Pull Request:\s*.*$");
        re.replace(message, format!("Pull Request: #{}", pr_number))
            .to_string()
    } else {
        // Add PR number at the end
        let trimmed = message.trim();
        if trimmed.is_empty() {
            format!("Pull Request: #{}", pr_number)
        } else {
            format!("{}\n\nPull Request: #{}", trimmed, pr_number)
        }
    }
}

/// Remove PR number from commit message
pub fn remove_pr_number(message: &str) -> String {
    let re = regex!(r"(?im)^Pull Request:\s*.*$\n?");
    re.replace_all(message, "").trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_pr_number() {
        assert_eq!(
            parse_pr_number("Add feature\n\nSome description\n\nPull Request: #123"),
            Some(123)
        );

        assert_eq!(
            parse_pr_number("Add feature\n\nPull Request: https://github.com/owner/repo/pull/456"),
            Some(456)
        );

        assert_eq!(
            parse_pr_number("Add feature\n\nNo PR here"),
            None
        );
    }

    #[test]
    fn test_add_pr_number() {
        let msg = "Add feature\n\nSome description";
        let result = add_pr_number(msg, 123);
        assert!(result.contains("Pull Request: #123"));
        assert_eq!(parse_pr_number(&result), Some(123));

        // Test updating existing PR number
        let result2 = add_pr_number(&result, 456);
        assert!(result2.contains("Pull Request: #456"));
        assert!(!result2.contains("#123"));
    }

    #[test]
    fn test_remove_pr_number() {
        let msg = "Add feature\n\nSome description\n\nPull Request: #123";
        let result = remove_pr_number(msg);
        assert!(!result.contains("Pull Request"));
        assert_eq!(result, "Add feature\n\nSome description");
    }
}
