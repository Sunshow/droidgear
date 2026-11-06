//! OMP channel import: build one `models.yml` provider entry from a channel.
//!
//! The inference used to live only in the GUI dialogs, which left the TUI
//! unable to import a channel the same way. It is pure (no disk or network
//! access) so the GUI dialog, the TUI action and any future CLI share it.

use serde::{Deserialize, Serialize};
use specta::Type;
use std::collections::HashMap;
use std::path::Path;

use crate::channel_import::{
    infer_api, normalize_base_url_for_channel, sanitize_model_ids, suggest_provider_id,
    DEFAULT_CONTEXT_WINDOW, DEFAULT_MAX_TOKENS,
};
use crate::omp::{OmpCustomModel, OmpCustomProvider};

/// One model listed by a channel token. `name` is the display name the channel
/// returned; the vendored registry wins when it knows the id.
#[derive(Debug, Clone, Default, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct OmpChannelModel {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// The resolved channel/token pair plus the models the caller selected.
///
/// Channel and model discovery stays in the caller (GUI command or TUI action)
/// so this module never talks to the network.
#[derive(Debug, Clone, Default, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct OmpChannelImportInput {
    pub channel_name: String,
    #[serde(default)]
    pub channel_type: String,
    pub base_url: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub platform: Option<String>,
    pub api_key: String,
    #[serde(default)]
    pub token_name: String,
    #[serde(default)]
    pub provider_id: String,
    #[serde(default)]
    pub model_ids: Vec<String>,
    #[serde(default)]
    pub channel_models: Vec<OmpChannelModel>,
    #[serde(default)]
    pub existing_provider_ids: Vec<String>,
    /// The live provider entry when re-importing into an existing id.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub existing: Option<OmpCustomProvider>,
}

/// The provider entry the caller saves.
#[derive(Debug, Clone, Default, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct OmpChannelImportResult {
    pub provider_id: String,
    pub provider: OmpCustomProvider,
}

/// Build the OMP `models.yml` provider entry for a channel import.
pub fn prepare_omp_channel_import(
    input: &OmpChannelImportInput,
) -> Result<OmpChannelImportResult, String> {
    let api_key = input.api_key.trim();
    if api_key.is_empty() {
        return Err("channel import needs an API key".to_string());
    }

    let from_models: Vec<String> = input
        .channel_models
        .iter()
        .map(|model| model.id.trim().to_string())
        .collect();
    let mut ids = sanitize_model_ids(&input.model_ids);
    if ids.is_empty() {
        ids = sanitize_model_ids(&from_models);
    }
    if ids.is_empty() {
        // A re-import with no selected models keeps the provider's current
        // models and only refreshes credentials/protocol.
        if let Some(existing) = input.existing.as_ref() {
            let existing_ids: Vec<String> = existing
                .models
                .iter()
                .map(|model| model.id.clone())
                .collect();
            ids = sanitize_model_ids(&existing_ids);
        }
    }
    if ids.is_empty() {
        return Err("select at least one model to import".to_string());
    }

    let provider_id = if input.provider_id.trim().is_empty() {
        suggest_provider_id(
            &input.channel_name,
            &input.token_name,
            &input.existing_provider_ids,
        )
    } else {
        input.provider_id.trim().to_string()
    };

    let display_names: HashMap<String, Option<String>> = input
        .channel_models
        .iter()
        .map(|model| (model.id.clone(), model.name.clone()))
        .collect();
    let imported: Vec<OmpCustomModel> = ids
        .iter()
        .map(|id| {
            let display = display_names.get(id.as_str()).and_then(|name| name.clone());
            omp_model_from_channel(id, display.as_deref())
        })
        .collect();

    let mut provider = input.existing.clone().unwrap_or_default();
    provider.base_url = Some(normalize_base_url_for_channel(
        &input.channel_type,
        &input.base_url,
        input.platform.as_deref(),
    ));
    provider.api = Some(infer_api(input.platform.as_deref(), &ids[0]));
    provider.api_key = Some(api_key.to_string());
    if provider
        .auth
        .as_deref()
        .map(str::trim)
        .unwrap_or("")
        .is_empty()
    {
        provider.auth = Some("apiKey".to_string());
    }
    provider.models = merge_omp_models(&provider.models, &imported);
    crate::omp::validate_omp_custom_provider(&provider)?;
    Ok(OmpChannelImportResult {
        provider_id,
        provider,
    })
}

/// Enrich a channel model from the vendored registry (the previous GUI-only
/// logic, now shared).
fn omp_model_from_channel(id: &str, display_name: Option<&str>) -> OmpCustomModel {
    let entry = crate::pi::find_registry_model(id);
    let mut name = display_name
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string);
    if let Some(entry) = entry {
        if !entry.name.trim().is_empty() {
            name = Some(entry.name.clone());
        }
    }
    let mut input = None;
    let mut reasoning = false;
    if let Some(entry) = entry {
        reasoning = entry.reasoning;
        if !entry.input.is_empty() {
            input = Some(entry.input.clone());
        }
    }
    let context_window = entry
        .map(|entry| entry.context_window)
        .filter(|window| *window != 0)
        .unwrap_or(DEFAULT_CONTEXT_WINDOW);
    let max_tokens = entry
        .and_then(|entry| entry.max_output_tokens)
        .filter(|tokens| *tokens != 0)
        .unwrap_or(DEFAULT_MAX_TOKENS);
    OmpCustomModel {
        id: id.to_string(),
        name,
        reasoning: Some(reasoning),
        input,
        context_window: Some(context_window),
        max_tokens: Some(max_tokens),
        ..Default::default()
    }
}

/// Keep the existing models (and their unknown fields) and fold the imported
/// specs over them by id.
fn merge_omp_models(
    existing: &[OmpCustomModel],
    imported: &[OmpCustomModel],
) -> Vec<OmpCustomModel> {
    let mut by_id: HashMap<String, OmpCustomModel> = HashMap::new();
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
        by_id.insert(model.id.clone(), merge_omp_model(current.as_ref(), model));
    }
    order
        .into_iter()
        .filter_map(|id| by_id.remove(&id))
        .collect()
}

fn merge_omp_model(existing: Option<&OmpCustomModel>, imported: &OmpCustomModel) -> OmpCustomModel {
    let Some(existing) = existing else {
        return imported.clone();
    };
    let mut out = existing.clone();
    out.id = imported.id.clone();
    if imported.name.is_some() {
        out.name = imported.name.clone();
    }
    if imported.api.is_some() {
        out.api = imported.api.clone();
    }
    if imported.base_url.is_some() {
        out.base_url = imported.base_url.clone();
    }
    if imported.reasoning.is_some() {
        out.reasoning = imported.reasoning;
    }
    if imported.thinking.is_some() {
        out.thinking = imported.thinking.clone();
    }
    if imported.input.is_some() {
        out.input = imported.input.clone();
    }
    if imported.image_input_decoder.is_some() {
        out.image_input_decoder = imported.image_input_decoder.clone();
    }
    if imported.cost.is_some() {
        out.cost = imported.cost.clone();
    }
    if imported.context_window.is_some() {
        out.context_window = imported.context_window;
    }
    if imported.max_context_window.is_some() {
        out.max_context_window = imported.max_context_window;
    }
    if imported.max_tokens.is_some() {
        out.max_tokens = imported.max_tokens;
    }
    if imported.omit_max_output_tokens.is_some() {
        out.omit_max_output_tokens = imported.omit_max_output_tokens;
    }
    if imported.prefer_websockets.is_some() {
        out.prefer_websockets = imported.prefer_websockets;
    }
    if imported.headers.is_some() {
        out.headers = imported.headers.clone();
    }
    if imported.compat.is_some() {
        out.compat = imported.compat.clone();
    }
    if imported.remote_compaction.is_some() {
        out.remote_compaction = imported.remote_compaction.clone();
    }
    out
}

/// Build and write one imported provider into the live models.yml, keeping every
/// other provider and unknown key. Returns the provider entry plus the role
/// repair the write triggered.
pub fn import_omp_channel_provider_for_home(
    home_dir: &Path,
    input: &OmpChannelImportInput,
) -> Result<(OmpChannelImportResult, crate::omp_roles::OmpRoleReconcile), String> {
    let result = prepare_omp_channel_import(input)?;
    let reconcile = crate::omp::save_omp_custom_provider_for_home(
        home_dir,
        &result.provider_id,
        result.provider.clone(),
    )?;
    Ok((result, reconcile))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prepare_enriches_models_and_keeps_existing_settings() {
        let existing = OmpCustomProvider {
            guardrail_identifier: Some("keep-me".to_string()),
            models: vec![OmpCustomModel {
                id: "keep-model".to_string(),
                name: Some("Keep".to_string()),
                ..Default::default()
            }],
            ..Default::default()
        };
        let input = OmpChannelImportInput {
            channel_name: "My Channel".to_string(),
            channel_type: "new-api".to_string(),
            base_url: "https://api.test".to_string(),
            platform: Some("openai".to_string()),
            api_key: "sk-test".to_string(),
            token_name: String::new(),
            provider_id: String::new(),
            model_ids: vec!["gpt-6-luna".to_string()],
            channel_models: vec![],
            existing_provider_ids: vec![],
            existing: Some(existing),
        };

        let result = prepare_omp_channel_import(&input).expect("prepare");
        assert_eq!(result.provider_id, "my-channel");
        assert_eq!(
            result.provider.base_url.as_deref(),
            Some("https://api.test/v1")
        );
        assert_eq!(
            result.provider.api.as_deref(),
            Some(crate::channel_import::API_OPENAI_COMPLETIONS)
        );
        assert_eq!(result.provider.auth.as_deref(), Some("apiKey"));
        assert_eq!(
            result.provider.guardrail_identifier.as_deref(),
            Some("keep-me")
        );
        let ids: Vec<&str> = result
            .provider
            .models
            .iter()
            .map(|model| model.id.as_str())
            .collect();
        assert_eq!(ids, vec!["keep-model", "gpt-6-luna"]);
    }

    #[test]
    fn prepare_rejects_a_missing_key_and_empty_selection() {
        let mut input = OmpChannelImportInput {
            channel_name: "c".to_string(),
            base_url: "https://api.test".to_string(),
            api_key: "  ".to_string(),
            model_ids: vec!["m".to_string()],
            ..Default::default()
        };
        assert!(prepare_omp_channel_import(&input).is_err());

        input.api_key = "sk".to_string();
        input.model_ids.clear();
        assert!(prepare_omp_channel_import(&input).is_err());
    }
}
