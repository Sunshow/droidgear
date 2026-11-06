//! Lossless merge of a Pi profile over the live `~/.pi/agent/models.json`.
//!
//! Applying a profile used to rebuild models.json from the profile alone, which
//! dropped every field this build does not model and the optional `$schema`
//! root key. The merge below keeps the live document as the base and folds the
//! profile's providers over it, so upstream additions survive an apply.

use std::collections::HashMap;

use crate::pi::{PiCompatConfig, PiModel, PiModelOverride, PiProviderConfig};

/// Merge the profile's providers into the live models.json document.
///
/// Unknown root keys and `$schema` are kept. With `keep_others == false`
/// (apply) providers absent from the profile are removed — the profile is the
/// desired state; with `keep_others == true` (single-provider import) they are
/// preserved. Matching providers keep their unknown keys and model-level
/// extras.
pub fn merge_pi_config(
    live: &str,
    providers: &HashMap<String, PiProviderConfig>,
    keep_others: bool,
) -> Result<String, String> {
    let mut root: serde_json::Map<String, serde_json::Value> = if live.trim().is_empty() {
        serde_json::Map::new()
    } else {
        serde_json::from_str(live).map_err(|e| format!("Invalid Pi config JSON: {e}"))?
    };

    let live_providers = providers_from_root(&root)?;

    let mut merged: HashMap<String, PiProviderConfig> = HashMap::new();
    if keep_others {
        for (id, provider) in &live_providers {
            merged.insert(id.clone(), provider.clone());
        }
    }
    for (id, provider) in providers {
        merged.insert(
            id.clone(),
            merge_pi_provider(live_providers.get(id), provider),
        );
    }

    let raw = serde_json::to_value(&merged)
        .map_err(|e| format!("Failed to serialize Pi providers: {e}"))?;
    root.insert("providers".to_string(), raw);

    serde_json::to_string_pretty(&serde_json::Value::Object(root))
        .map_err(|e| format!("Failed to serialize Pi config: {e}"))
}

/// Parse the `providers` object of a models.json document.
pub fn providers_from_document(live: &str) -> Result<HashMap<String, PiProviderConfig>, String> {
    if live.trim().is_empty() {
        return Ok(HashMap::new());
    }
    let root: serde_json::Map<String, serde_json::Value> =
        serde_json::from_str(live).map_err(|e| format!("Invalid Pi config JSON: {e}"))?;
    providers_from_root(&root)
}

fn providers_from_root(
    root: &serde_json::Map<String, serde_json::Value>,
) -> Result<HashMap<String, PiProviderConfig>, String> {
    match root.get("providers") {
        None => Ok(HashMap::new()),
        Some(raw) => serde_json::from_value(raw.clone())
            .map_err(|e| format!("Invalid Pi providers object: {e}")),
    }
}

/// Fold a profile provider over the live one of the same id.
///
/// Unknown provider-level keys and model-level extras survive; a field the
/// profile leaves empty keeps the live value (oauth, headers, auth header,
/// compat). Known provider fields otherwise come from the profile.
pub fn merge_pi_provider(
    existing: Option<&PiProviderConfig>,
    incoming: &PiProviderConfig,
) -> PiProviderConfig {
    let Some(existing) = existing else {
        return incoming.clone();
    };

    let mut out = incoming.clone();
    for (key, value) in &existing.extra {
        out.extra
            .entry(key.clone())
            .or_insert_with(|| value.clone());
    }
    if out.oauth.is_none() {
        out.oauth = existing.oauth.clone();
    }
    if out.headers.is_none() {
        out.headers = existing.headers.clone();
    }
    if out.auth_header.is_none() {
        out.auth_header = existing.auth_header;
    }
    if out.name.is_none() {
        out.name = existing.name.clone();
    }
    if out.compat.is_none() {
        out.compat = existing.compat.clone();
    }
    out.models = merge_pi_models(&existing.models, &out.models);
    out.model_overrides = merge_pi_model_overrides(
        existing.model_overrides.as_ref(),
        out.model_overrides.take(),
    );
    out
}

/// Keep the provider's models and fold the incoming specs over them by id.
pub fn merge_pi_models(existing: &[PiModel], imported: &[PiModel]) -> Vec<PiModel> {
    let mut by_id: HashMap<String, PiModel> = HashMap::new();
    let mut order: Vec<String> = Vec::new();
    for model in existing {
        if !by_id.contains_key(&model.id) {
            order.push(model.id.clone());
        }
        by_id.insert(model.id.clone(), model.clone());
    }
    for model in imported {
        let current = by_id.get(&model.id).cloned();
        if current.is_none() {
            order.push(model.id.clone());
        }
        by_id.insert(model.id.clone(), merge_pi_model(current.as_ref(), model));
    }
    order
        .into_iter()
        .filter_map(|id| by_id.remove(&id))
        .collect()
}

/// Fold an imported model over the existing one of the same id.
fn merge_pi_model(existing: Option<&PiModel>, imported: &PiModel) -> PiModel {
    let Some(existing) = existing else {
        return imported.clone();
    };

    let mut out = existing.clone();
    out.id = imported.id.clone();
    if imported.model_type.is_some() {
        out.model_type = imported.model_type.clone();
    }
    if imported.name.is_some() {
        out.name = imported.name.clone();
    }
    if imported.api.is_some() {
        out.api = imported.api.clone();
    }
    if imported.base_url.is_some() {
        out.base_url = imported.base_url.clone();
    }
    out.reasoning = imported.reasoning;
    if !imported.input.is_empty() {
        out.input = imported.input.clone();
    }
    if imported.input_limits.is_some() {
        out.input_limits = imported.input_limits.clone();
    }
    if imported.thinking_level_map.is_some() {
        out.thinking_level_map = imported.thinking_level_map.clone();
    }
    if imported.context_window != 0 {
        out.context_window = imported.context_window;
    }
    if imported.max_tokens != 0 {
        out.max_tokens = imported.max_tokens;
    }
    if imported.prompt_cache.is_some() {
        out.prompt_cache = imported.prompt_cache.clone();
    }
    if imported.sampling_params.is_some() {
        out.sampling_params = imported.sampling_params.clone();
    }
    if imported.sampling_params_by_thinking_level.is_some() {
        out.sampling_params_by_thinking_level = imported.sampling_params_by_thinking_level.clone();
    }
    out.compat = merge_pi_compat(out.compat.take(), imported.compat.as_ref());
    out
}

/// Overlay the imported compat only where the existing compat leaves a field
/// unset, so an explicit user choice always wins.
fn merge_pi_compat(
    existing: Option<PiCompatConfig>,
    imported: Option<&PiCompatConfig>,
) -> Option<PiCompatConfig> {
    let imported = imported?;
    let Some(mut existing) = existing else {
        return Some(imported.clone());
    };

    macro_rules! fill {
        ($field:ident) => {
            if existing.$field.is_none() {
                existing.$field = imported.$field.clone();
            }
        };
    }
    fill!(supports_store);
    fill!(supports_developer_role);
    fill!(supports_reasoning_effort);
    fill!(reasoning_effort_map);
    fill!(supports_usage_in_streaming);
    fill!(max_tokens_field);
    fill!(requires_tool_result_name);
    fill!(requires_assistant_after_tool_result);
    fill!(requires_thinking_as_text);
    fill!(requires_reasoning_content_on_assistant_messages);
    fill!(thinking_format);
    fill!(cache_control_format);
    fill!(supports_strict_mode);
    fill!(supports_long_cache_retention);
    fill!(supports_eager_tool_input_streaming);
    fill!(force_adaptive_thinking);
    fill!(open_router_routing);
    fill!(vercel_gateway_routing);
    fill!(supports_finish_reason);
    fill!(requires_non_null_assistant_content);
    fill!(chat_template_kwargs);
    fill!(chat_template_args);
    fill!(zai_tool_stream);
    fill!(thinking_token_budget_field);
    fill!(supports_thinking_token_budget);
    fill!(supports_openai_grammar_tools);
    fill!(supports_mid_convo_system_messages);
    fill!(supports_mid_convo_tool_additions);
    fill!(send_session_affinity_headers);
    fill!(session_affinity_format);
    fill!(prompt_cache_key_mode);
    fill!(vllm_priority);
    fill!(supports_additional_tools);
    fill!(supports_tool_search);
    fill!(supports_explicit_prompt_cache_mode);
    fill!(supports_mid_convo_effort);
    fill!(supports_max_output_tokens);
    fill!(supports_cache_control_on_tools);
    fill!(supports_temperature);
    fill!(allow_empty_signature);
    fill!(supports_strict_tools);
    fill!(supports_mid_convo_tool_changes);
    fill!(supports_message_threads);
    fill!(allowed_fallback_models);
    for (key, value) in &imported.extra {
        existing
            .extra
            .entry(key.clone())
            .or_insert_with(|| value.clone());
    }
    Some(existing)
}

/// Unknown override keys survive a provider merge.
fn merge_pi_model_overrides(
    existing: Option<&HashMap<String, PiModelOverride>>,
    incoming: Option<HashMap<String, PiModelOverride>>,
) -> Option<HashMap<String, PiModelOverride>> {
    let mut incoming = incoming?;
    let Some(existing) = existing else {
        return Some(incoming);
    };
    for (id, override_model) in incoming.iter_mut() {
        if let Some(previous) = existing.get(id) {
            for (key, value) in &previous.extra {
                override_model
                    .extra
                    .entry(key.clone())
                    .or_insert_with(|| value.clone());
            }
        }
    }
    Some(incoming)
}

/// Read the live models.json bytes (empty when the file does not exist).
pub fn read_live_config_for_merge(path: &std::path::Path) -> Result<String, String> {
    match std::fs::read_to_string(path) {
        Ok(data) => Ok(data),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(String::new()),
        Err(e) => Err(format!("Failed to read Pi config file: {e}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn model(id: &str) -> PiModel {
        PiModel {
            id: id.to_string(),
            ..Default::default()
        }
    }

    #[test]
    fn merge_keeps_unknown_provider_and_model_keys() {
        let live = r#"{
          "$schema": "https://example.test/schema.json",
          "providers": {
            "acme": {
              "baseUrl": "https://old.test",
              "futureProviderKey": 7,
              "models": [{"id": "m1", "futureModelKey": "keep", "contextWindow": 1000}]
            }
          }
        }"#;
        let mut models = vec![model("m1")];
        models[0].context_window = 2000;
        let providers = HashMap::from([(
            "acme".to_string(),
            PiProviderConfig {
                api: Some("openai-completions".to_string()),
                models,
                ..Default::default()
            },
        )]);

        let merged = merge_pi_config(live, &providers, false).expect("merge");
        let value: serde_json::Value = serde_json::from_str(&merged).expect("json");

        assert_eq!(
            value.get("$schema").and_then(|v| v.as_str()),
            Some("https://example.test/schema.json")
        );
        let provider = &value["providers"]["acme"];
        assert_eq!(
            provider.get("futureProviderKey").and_then(|v| v.as_i64()),
            Some(7)
        );
        let model_value = &provider["models"][0];
        assert_eq!(
            model_value.get("futureModelKey").and_then(|v| v.as_str()),
            Some("keep")
        );
        assert_eq!(
            model_value.get("contextWindow").and_then(|v| v.as_u64()),
            Some(2000)
        );
    }

    #[test]
    fn merge_drops_providers_absent_from_the_profile() {
        let live = r#"{"providers": {"old": {"baseUrl": "https://old.test"}}}"#;
        let providers = HashMap::from([(
            "new".to_string(),
            PiProviderConfig {
                base_url: Some("https://new.test".to_string()),
                models: vec![model("m1")],
                ..Default::default()
            },
        )]);

        let merged = merge_pi_config(live, &providers, false).expect("merge");
        let value: serde_json::Value = serde_json::from_str(&merged).expect("json");
        let providers = value["providers"].as_object().expect("providers");
        assert!(!providers.contains_key("old"));
        assert!(providers.contains_key("new"));
    }

    #[test]
    fn merge_keeps_other_providers_when_importing_one() {
        let live = r#"{"providers": {"old": {"baseUrl": "https://old.test"}}}"#;
        let providers = HashMap::from([(
            "new".to_string(),
            PiProviderConfig {
                models: vec![model("m1")],
                ..Default::default()
            },
        )]);

        let merged = merge_pi_config(live, &providers, true).expect("merge");
        let value: serde_json::Value = serde_json::from_str(&merged).expect("json");
        let providers = value["providers"].as_object().expect("providers");
        assert!(providers.contains_key("old"));
        assert!(providers.contains_key("new"));
    }
}
