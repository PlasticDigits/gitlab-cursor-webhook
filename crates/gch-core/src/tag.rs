// SPDX-License-Identifier: AGPL-3.0-or-later

/// GitLab label prefix for issue-triggered agent flows (`agent:verify` → tag `verify`).
pub const LABEL_AGENT_PREFIX: &str = "agent:";

/// MR-only security reviewer tag (not triggerable via `agent:security` on issues).
pub const MR_SECURITY_TAG: &str = "security";

/// Tag names that may only be triggered by reserved webhook handlers (not `agent:*` labels).
pub const ISSUE_LABEL_RESERVED_TAGS: &[&str] = &[MR_SECURITY_TAG];

/// Higher-priority issue tags when multiple `agent:*` labels fire on one event.
pub const ISSUE_TAG_PRIORITY: &[&str] = &["implement", "verify"];

pub fn validate_tag_name(name: &str) -> Result<(), String> {
    let name = name.trim();
    if name.is_empty() {
        return Err("tag name must not be empty".into());
    }
    if !name
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
    {
        return Err(format!(
            "invalid tag name {name:?}: use lowercase letters, digits, underscores"
        ));
    }
    if !name
        .chars()
        .next()
        .is_some_and(|c| c.is_ascii_lowercase())
    {
        return Err(format!("invalid tag name {name:?}: must start with a letter"));
    }
    Ok(())
}

pub fn is_issue_label_reserved(tag: &str) -> bool {
    ISSUE_LABEL_RESERVED_TAGS.contains(&tag)
}

/// Map `agent:{tag}` label to tag name, skipping reserved and malformed names.
pub fn tag_from_agent_label(label: &str) -> Option<String> {
    let suffix = label.strip_prefix(LABEL_AGENT_PREFIX)?;
    if suffix.is_empty() || is_issue_label_reserved(suffix) {
        return None;
    }
    validate_tag_name(suffix).ok()?;
    Some(suffix.to_string())
}

/// Order triggered issue tags: priority list first, then remaining alphabetically.
pub fn order_issue_tags(tags: &[String]) -> Vec<String> {
    let mut ordered = ISSUE_TAG_PRIORITY
        .iter()
        .filter_map(|preferred| tags.iter().find(|t| t.as_str() == *preferred).cloned())
        .collect::<Vec<_>>();
    let mut rest: Vec<String> = tags
        .iter()
        .filter(|t| !ISSUE_TAG_PRIORITY.contains(&t.as_str()))
        .cloned()
        .collect();
    rest.sort();
    ordered.extend(rest);
    ordered
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_accepts_configured_tags() {
        for name in ["security", "verify", "implement", "gap_analysis", "security_audit"] {
            assert!(validate_tag_name(name).is_ok(), "{name}");
        }
    }

    #[test]
    fn validate_rejects_invalid_names() {
        assert!(validate_tag_name("").is_err());
        assert!(validate_tag_name("Gap").is_err());
        assert!(validate_tag_name("1bad").is_err());
        assert!(validate_tag_name("has-dash").is_err());
    }

    #[test]
    fn security_is_reserved_for_issue_labels() {
        assert!(is_issue_label_reserved("security"));
        assert!(!is_issue_label_reserved("verify"));
        assert_eq!(tag_from_agent_label("agent:security"), None);
        assert_eq!(
            tag_from_agent_label("agent:gap_analysis").as_deref(),
            Some("gap_analysis")
        );
    }

    #[test]
    fn order_issue_tags_applies_priority_then_alpha() {
        let tags = vec![
            "gap_analysis".into(),
            "verify".into(),
            "implement".into(),
            "security_audit".into(),
        ];
        assert_eq!(
            order_issue_tags(&tags),
            vec![
                "implement".to_string(),
                "verify".to_string(),
                "gap_analysis".to_string(),
                "security_audit".to_string(),
            ]
        );
    }
}
