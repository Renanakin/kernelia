use serde::{Deserialize, Serialize};

pub const MS_ALLOWED_DOMAINS: &[&str] = &[
    "learn.microsoft.com",
    "support.microsoft.com",
    "answers.microsoft.com",
    "techcommunity.microsoft.com",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MicrosoftSearchResult {
    pub title: String,
    pub snippet: String,
    pub url: String,
    pub domain: String,
}

pub fn is_whitelisted_microsoft_url(url: &str) -> bool {
    let lowercase = url.to_lowercase();
    MS_ALLOWED_DOMAINS.iter().any(|d| lowercase.contains(d))
}

pub fn build_microsoft_search_query(user_query: &str) -> String {
    let clean_query = user_query
        .replace("site:", "")
        .replace("http", "")
        .trim()
        .to_string();
    
    format!(
        "{} (site:learn.microsoft.com OR site:support.microsoft.com OR site:answers.microsoft.com OR site:techcommunity.microsoft.com)",
        clean_query
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_whitelisted_microsoft_domains() {
        assert!(is_whitelisted_microsoft_url("https://learn.microsoft.com/en-us/windows/"));
        assert!(is_whitelisted_microsoft_url("https://support.microsoft.com/kb/12345"));
        assert!(is_whitelisted_microsoft_url("https://answers.microsoft.com/thread/99"));
        assert!(!is_whitelisted_microsoft_url("https://untrusted-thirdparty-forum.com/malware"));
    }

    #[test]
    fn builds_whitelisted_search_query() {
        let query = build_microsoft_search_query("error spooler windows 11");
        assert!(query.contains("site:learn.microsoft.com"));
        assert!(query.contains("error spooler windows 11"));
    }
}
