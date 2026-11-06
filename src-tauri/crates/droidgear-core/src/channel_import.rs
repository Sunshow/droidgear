//! Pure helpers shared by the Pi and OMP channel-import flows: provider-id
//! suggestion, base-URL normalization and API-protocol inference.
//!
//! The rules used to live in the React dialogs
//! (`src/components/{pi,omp}/*ImportFromChannel*`), which meant the GUI was the
//! only consumer and the TUI could not import a channel the same way. They are
//! pure string/registry transformations, so they belong in the core where every
//! consumer shares one implementation.

/// API protocol identifiers written to Pi `models.json` / OMP `models.yml`.
pub const API_OPENAI_COMPLETIONS: &str = "openai-completions";
pub const API_OPENAI_RESPONSES: &str = "openai-responses";
pub const API_ANTHROPIC_MESSAGES: &str = "anthropic-messages";
pub const API_GOOGLE_GENERATIVE_AI: &str = "google-generative-ai";

/// Defaults used when the vendored registry has no entry for a channel model.
pub const DEFAULT_CONTEXT_WINDOW: u32 = 128_000;
pub const DEFAULT_MAX_TOKENS: u32 = 16_384;

/// Make a config-key-safe provider id out of a display name. The result can be
/// empty; the caller must handle that.
pub fn sanitize_provider_id(raw: &str) -> String {
    let mut id = String::with_capacity(raw.len());
    let mut last_dash = false;
    for ch in raw.to_lowercase().chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' {
            id.push(ch);
            last_dash = ch == '-';
        } else if !last_dash {
            id.push('-');
            last_dash = true;
        }
    }
    id.trim_matches('-').to_string()
}

/// Derive a non-conflicting provider id from the channel (and optional token)
/// name, mirroring the GUI dialogs' suggestion rule.
pub fn suggest_provider_id(channel_name: &str, token_name: &str, existing: &[String]) -> String {
    let base = if token_name.trim().is_empty() {
        channel_name.to_string()
    } else {
        format!("{channel_name}-{token_name}")
    };
    let base = sanitize_provider_id(&base);
    let base = if base.is_empty() {
        "imported-provider".to_string()
    } else {
        base
    };
    if !existing.iter().any(|id| id == &base) {
        return base;
    }
    let mut index = 2;
    loop {
        let candidate = format!("{base}-{index}");
        if !existing.iter().any(|id| id == &candidate) {
            return candidate;
        }
        index += 1;
    }
}

/// Append `suffix` unless the trimmed URL already ends with it (mirrors the
/// frontend `normalizeBaseUrl` helper).
pub fn normalize_base_url(base_url: &str, suffix: &str) -> String {
    let trimmed = base_url.trim_end_matches('/');
    if suffix.is_empty() || trimmed.ends_with(suffix) {
        trimmed.to_string()
    } else {
        format!("{trimmed}{suffix}")
    }
}

/// Append `/v1` to an OpenAI-compatible endpoint unless it is empty or already
/// carries a version segment (`/v1`, `/v1beta` …).
pub fn ensure_openai_compatible_v1(base_url: &str) -> String {
    let trimmed = base_url.trim();
    if trimmed.is_empty() || has_version_path(trimmed) {
        return trimmed.to_string();
    }
    format!("{}/v1", trimmed.trim_end_matches('/'))
}

fn has_version_path(url: &str) -> bool {
    let bytes: Vec<char> = url.chars().collect();
    bytes
        .windows(3)
        .any(|window| window[0] == '/' && window[1] == 'v' && window[2].is_ascii_digit())
}

/// Chat Completions endpoints behind sub-2-api channels need the `/v1` suffix.
pub fn needs_openai_compatible_v1(channel_type: &str, uses_chat_completions: bool) -> bool {
    channel_type == "sub-2-api" && uses_chat_completions
}

/// Map a channel platform to the wire protocol Pi / OMP should use. An untagged
/// platform is treated as OpenAI Chat Completions; antigravity is decided per
/// model.
pub fn infer_api(platform: Option<&str>, model_id: &str) -> String {
    match platform.unwrap_or("").trim().to_lowercase().as_str() {
        "anthropic" | "claude" => API_ANTHROPIC_MESSAGES.to_string(),
        "gemini" => API_GOOGLE_GENERATIVE_AI.to_string(),
        "antigravity" => {
            if model_id.to_lowercase().starts_with("claude-") {
                API_ANTHROPIC_MESSAGES.to_string()
            } else {
                API_OPENAI_COMPLETIONS.to_string()
            }
        }
        _ => API_OPENAI_COMPLETIONS.to_string(),
    }
}

fn is_platform(platform: Option<&str>, want: &str) -> bool {
    platform.is_some_and(|value| value.trim().eq_ignore_ascii_case(want))
}

/// The shared Pi/OMP rule: Chat Completions endpoints behind sub-2-api channels
/// and generic compatible routes get the `/v1` suffix; OpenAI-platform routes
/// are normalized to `/v1`; Anthropic, Gemini and antigravity keep their channel
/// base URL.
pub fn normalize_base_url_for_channel(
    channel_type: &str,
    base_url: &str,
    platform: Option<&str>,
) -> String {
    let api = infer_api(platform, "");
    if !is_platform(platform, "antigravity")
        && needs_openai_compatible_v1(channel_type, api == API_OPENAI_COMPLETIONS)
    {
        return ensure_openai_compatible_v1(base_url);
    }
    if platform.map(str::trim).unwrap_or("").is_empty() || is_platform(platform, "openai") {
        return normalize_base_url(base_url, "/v1");
    }
    base_url.trim_end_matches('/').to_string()
}

/// Drop blank entries and duplicates while keeping the caller's order (selected
/// models arrive from a UI checkbox set).
pub fn sanitize_model_ids(ids: &[String]) -> Vec<String> {
    let mut out: Vec<String> = Vec::with_capacity(ids.len());
    for id in ids {
        let id = id.trim();
        if id.is_empty() || out.iter().any(|seen| seen == id) {
            continue;
        }
        out.push(id.to_string());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_id_suggestion_avoids_conflicts() {
        assert_eq!(
            suggest_provider_id("My Channel", "", &[]),
            "my-channel".to_string()
        );
        assert_eq!(
            suggest_provider_id("My Channel", "token 1", &[]),
            "my-channel-token-1".to_string()
        );
        let existing = vec!["my-channel".to_string(), "my-channel-2".to_string()];
        assert_eq!(
            suggest_provider_id("My Channel", "", &existing),
            "my-channel-3".to_string()
        );
        assert_eq!(suggest_provider_id("!!!", "", &[]), "imported-provider");
    }

    #[test]
    fn base_url_normalization_follows_the_channel_rules() {
        assert_eq!(
            normalize_base_url_for_channel("new-api", "https://api.test", Some("openai")),
            "https://api.test/v1"
        );
        assert_eq!(
            normalize_base_url_for_channel("new-api", "https://api.test", Some("anthropic")),
            "https://api.test"
        );
        assert_eq!(
            normalize_base_url_for_channel("sub-2-api", "https://api.test", Some("openai")),
            "https://api.test/v1"
        );
        assert_eq!(
            ensure_openai_compatible_v1("https://api.test/v1beta"),
            "https://api.test/v1beta"
        );
    }

    #[test]
    fn api_inference_maps_platforms() {
        assert_eq!(infer_api(Some("claude"), "x"), API_ANTHROPIC_MESSAGES);
        assert_eq!(infer_api(Some("gemini"), "x"), API_GOOGLE_GENERATIVE_AI);
        assert_eq!(
            infer_api(Some("antigravity"), "claude-opus-5-5"),
            API_ANTHROPIC_MESSAGES
        );
        assert_eq!(infer_api(None, "gpt-6-luna"), API_OPENAI_COMPLETIONS);
    }

    #[test]
    fn model_ids_are_sanitized_in_order() {
        let ids = vec![
            " a ".to_string(),
            "b".to_string(),
            "a".to_string(),
            "  ".to_string(),
        ];
        assert_eq!(sanitize_model_ids(&ids), vec!["a", "b"]);
    }
}
