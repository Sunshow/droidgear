//! Runtime environment for launching GitHub Copilot CLI.

use serde::{Deserialize, Serialize};
use specta::Type;

use crate::copilot::CopilotProfile;

const COPILOT_OFFLINE: &str = "COPILOT_OFFLINE";
const COPILOT_AUTH_TOKEN: &str = "COPILOT_AUTH_TOKEN";
const COPILOT_ENV_KEYS: [&str; 14] = [
    COPILOT_OFFLINE,
    super::copilot::COPILOT_PROVIDER_BASE_URL,
    super::copilot::COPILOT_PROVIDER_TYPE,
    super::copilot::COPILOT_PROVIDER_API_KEY,
    super::copilot::COPILOT_MODEL,
    super::copilot::COPILOT_PROVIDER_MAX_PROMPT_TOKENS,
    super::copilot::COPILOT_PROVIDER_MAX_OUTPUT_TOKENS,
    "COPILOT_PROVIDER_MODEL_ID",
    "COPILOT_PROVIDER_WIRE_MODEL",
    "COPILOT_PROVIDER_BEARER_TOKEN",
    "COPILOT_PROVIDER_API_KEY_COMMAND",
    "COPILOT_PROVIDER_HEADERS",
    "COPILOT_PROVIDER_WIRE_API",
    "COPILOT_PROVIDER_TRANSPORT",
];

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CopilotRunPlan {
    pub program: String,
    pub args: Vec<String>,
    pub env: Vec<(String, String)>,
    pub secret_env: Vec<(String, String)>,
    pub unset_env: Vec<String>,
}

fn trim_non_empty(value: Option<&str>) -> Option<&str> {
    value.map(str::trim).filter(|value| !value.is_empty())
}

/// Return public and secret env entries for a BYOK profile.
type EnvPairs = Vec<(String, String)>;

fn build_byok_env(profile: &CopilotProfile) -> (EnvPairs, EnvPairs) {
    let mut env = vec![
        (COPILOT_OFFLINE.to_string(), "true".to_string()),
        (
            super::copilot::COPILOT_PROVIDER_TYPE.to_string(),
            trim_non_empty(profile.provider_type.as_deref())
                .unwrap_or("openai")
                .to_string(),
        ),
    ];
    let mut secret_env = Vec::new();

    if let Some(value) = trim_non_empty(profile.base_url.as_deref()) {
        env.push((
            super::copilot::COPILOT_PROVIDER_BASE_URL.to_string(),
            value.to_string(),
        ));
    }
    if let Some(value) = trim_non_empty(profile.api_key.as_deref()) {
        secret_env.push((
            super::copilot::COPILOT_PROVIDER_API_KEY.to_string(),
            value.to_string(),
        ));
    }
    if let Some(value) = trim_non_empty(profile.model.as_deref()) {
        env.push((super::copilot::COPILOT_MODEL.to_string(), value.to_string()));
    }
    if let Some(value) = profile.max_prompt_tokens {
        env.push((
            super::copilot::COPILOT_PROVIDER_MAX_PROMPT_TOKENS.to_string(),
            value.to_string(),
        ));
    }
    if let Some(value) = profile.max_output_tokens {
        env.push((
            super::copilot::COPILOT_PROVIDER_MAX_OUTPUT_TOKENS.to_string(),
            value.to_string(),
        ));
    }

    (env, secret_env)
}

pub fn build_run_plan(profile: &CopilotProfile) -> Result<CopilotRunPlan, String> {
    let profile = super::copilot::normalize_profile(profile.clone())?;
    let mut unset_env = COPILOT_ENV_KEYS
        .iter()
        .map(|key| (*key).to_string())
        .collect::<Vec<_>>();
    let (env, secret_env) = if profile.use_official_auth {
        (Vec::new(), Vec::new())
    } else {
        let missing = [
            ("Base URL", profile.base_url.as_deref()),
            ("Model", profile.model.as_deref()),
            ("API key", profile.api_key.as_deref()),
        ]
        .into_iter()
        .filter_map(|(field, value)| trim_non_empty(value).is_none().then_some(field))
        .collect::<Vec<_>>();
        if !missing.is_empty() {
            return Err(format!(
                "Complete the Copilot BYOK profile before launching: {}",
                missing.join(", ")
            ));
        }
        unset_env.push(COPILOT_AUTH_TOKEN.to_string());
        build_byok_env(&profile)
    };

    Ok(CopilotRunPlan {
        program: "copilot".to_string(),
        // COPILOT_MODEL configures the provider; --model also overrides the
        // interactive UI's persisted model selection from the first render.
        args: if profile.use_official_auth {
            Vec::new()
        } else {
            vec![
                "--model".to_string(),
                profile
                    .model
                    .as_deref()
                    .unwrap_or_default()
                    .trim()
                    .to_string(),
            ]
        },
        env,
        secret_env,
        unset_env,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn profile() -> CopilotProfile {
        CopilotProfile {
            id: "id".to_string(),
            name: "name".to_string(),
            description: None,
            created_at: String::new(),
            updated_at: String::new(),
            use_official_auth: false,
            base_url: Some("https://example.test".to_string()),
            provider_type: Some("openai".to_string()),
            api_key: Some("key".to_string()),
            model: Some("model".to_string()),
            max_prompt_tokens: None,
            max_output_tokens: None,
        }
    }

    #[test]
    fn byok_secrets_are_separate_and_inherited_keys_are_cleared() {
        let plan = build_run_plan(&profile()).unwrap();
        assert_eq!(
            plan.secret_env,
            vec![("COPILOT_PROVIDER_API_KEY".to_string(), "key".to_string())]
        );
        assert!(plan.unset_env.contains(&"COPILOT_AUTH_TOKEN".to_string()));
        for key in [
            "COPILOT_PROVIDER_MODEL_ID",
            "COPILOT_PROVIDER_WIRE_MODEL",
            "COPILOT_PROVIDER_BEARER_TOKEN",
            "COPILOT_PROVIDER_API_KEY_COMMAND",
            "COPILOT_PROVIDER_HEADERS",
            "COPILOT_PROVIDER_WIRE_API",
            "COPILOT_PROVIDER_TRANSPORT",
        ] {
            assert!(
                plan.unset_env.iter().any(|value| value == key),
                "inherited {key} must be cleared"
            );
        }
    }

    #[test]
    fn run_selects_the_profile_model_explicitly_and_normalizes_unsaved_urls() {
        for (provider, input, expected) in [
            ("openai", "https://example.test", "https://example.test/v1"),
            (
                "anthropic",
                "https://example.test/v1",
                "https://example.test",
            ),
        ] {
            let mut value = profile();
            value.provider_type = Some(provider.to_string());
            value.base_url = Some(input.to_string());
            value.model = Some("  provider/custom-model  ".to_string());
            let plan = build_run_plan(&value).unwrap();
            assert_eq!(plan.args, vec!["--model", "provider/custom-model"]);
            assert!(plan.env.contains(&(
                "COPILOT_MODEL".to_string(),
                "provider/custom-model".to_string()
            )));
            assert!(plan.env.contains(&(
                "COPILOT_PROVIDER_BASE_URL".to_string(),
                expected.to_string()
            )));
            assert_eq!(value.base_url.as_deref(), Some(input));
        }
    }

    #[test]
    fn official_mode_does_not_inject_byok_values() {
        let mut value = profile();
        value.use_official_auth = true;
        let plan = build_run_plan(&value).unwrap();
        assert!(plan.env.is_empty());
        assert!(plan.secret_env.is_empty());
        assert!(plan.args.is_empty());
        assert!(!plan.unset_env.contains(&"COPILOT_AUTH_TOKEN".to_string()));
    }

    #[test]
    fn incomplete_byok_profile_cannot_launch() {
        let mut value = profile();
        value.model = Some("  ".to_string());
        assert!(build_run_plan(&value).unwrap_err().contains("Model"));
    }
}
