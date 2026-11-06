//! Keeping Pi's startup model resolvable across profile applies.
//!
//! `apply_pi_profile_for_home` replaces the providers `models.json` supplies,
//! but the startup model lives in `settings.json` (see [`crate::pi_settings`])
//! and pi resolves it against `models.json` **plus** its own catalog. Applying a
//! profile that no longer carries the provider — or renames a model — therefore
//! leaves `settings.json` pointing at something pi cannot start, and pi
//! silently falls back to another model.
//!
//! [`reconcile_startup_model_for_home`] is the one place that repairs that
//! selection. It is deliberately conservative: it only rewrites a value that an
//! apply made unresolvable. A selection served by pi's catalog
//! (`~/.pi/agent/models-store.json`) is never touched, because a profile says
//! nothing about catalog providers, and a provider that survives the apply keeps
//! its model unless the model itself is gone.

use serde::{Deserialize, Serialize};
use specta::Type;
use std::collections::HashMap;
use std::path::Path;

use crate::pi::{pi_config_dir_for_home, PiCurrentConfig, PiModel, PiProviderConfig};
use crate::pi_settings::{
    pi_settings_path_for_home, read_pi_settings_for_home, save_pi_settings_for_home, PiSettings,
    THINKING_LEVELS,
};

/// What one apply wrote: the providers now in models.json and the startup
/// selection that came out of the reconcile pass.
#[derive(Debug, Clone, Default, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct PiApplyResult {
    pub id: String,
    pub applied_providers: Vec<String>,
    pub startup: PiSettings,
    pub startup_adjusted: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub startup_reasons: Vec<String>,
}

/// What the startup model fields can point at, so the GUI pickers and the TUI
/// offer the same choices instead of asking users to type ids from memory.
#[derive(Debug, Clone, Default, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct StartupOptions {
    pub providers: Vec<String>,
    pub models: HashMap<String, Vec<String>>,
}

/// What pi's downloaded catalog knows about one model.
#[derive(Debug, Clone, Default)]
struct StartupCatalogModel {
    thinking_level_map: HashMap<String, Option<String>>,
}

/// What pi's downloaded catalog knows: provider id → model id.
type StartupCatalog = HashMap<String, HashMap<String, StartupCatalogModel>>;

fn knows_provider(catalog: &StartupCatalog, id: &str) -> bool {
    catalog.contains_key(id)
}

fn catalog_model<'a>(
    catalog: &'a StartupCatalog,
    provider_id: &str,
    model_id: &str,
) -> Option<&'a StartupCatalogModel> {
    catalog.get(provider_id)?.get(model_id)
}

/// The model's declared thinking level map when the catalog knows it.
fn catalog_thinking_levels(
    catalog: &StartupCatalog,
    provider_id: &str,
    model_id: &str,
) -> Option<HashMap<String, Option<String>>> {
    let entry = catalog_model(catalog, provider_id, model_id)?;
    if entry.thinking_level_map.is_empty() {
        return None;
    }
    Some(entry.thinking_level_map.clone())
}

/// Parse `models-store.json` (pi's cached catalog). A missing or malformed file
/// yields an empty catalog: the repair pass then only trusts models.json, which
/// keeps the historical behaviour for fresh installs.
fn read_startup_catalog_for_home(home_dir: &Path) -> StartupCatalog {
    let Ok(dir) = pi_config_dir_for_home(home_dir) else {
        return StartupCatalog::new();
    };
    let Ok(text) = std::fs::read_to_string(dir.join("models-store.json")) else {
        return StartupCatalog::new();
    };
    let Ok(store) = serde_json::from_str::<serde_json::Value>(&text) else {
        return StartupCatalog::new();
    };
    let mut catalog: StartupCatalog = HashMap::new();
    let Some(providers) = store.as_object() else {
        return catalog;
    };
    for (provider_id, entry) in providers {
        let mut models: HashMap<String, StartupCatalogModel> = HashMap::new();
        if let Some(list) = entry.get("models").and_then(|value| value.as_array()) {
            for model in list {
                let Some(id) = model.get("id").and_then(|value| value.as_str()) else {
                    continue;
                };
                if id.is_empty() {
                    continue;
                }
                let thinking_level_map = model
                    .get("thinkingLevelMap")
                    .and_then(|value| {
                        serde_json::from_value::<HashMap<String, Option<String>>>(value.clone())
                            .ok()
                    })
                    .unwrap_or_default();
                models.insert(id.to_string(), StartupCatalogModel { thinking_level_map });
            }
        }
        catalog.insert(provider_id.clone(), models);
    }
    catalog
}

fn optional_value(value: Option<&String>) -> String {
    value.map(|v| v.trim().to_string()).unwrap_or_default()
}

fn optional_value_ptr(value: &str) -> Option<String> {
    if value.is_empty() {
        None
    } else {
        Some(value.to_string())
    }
}

fn is_thinking_level(level: &str) -> bool {
    THINKING_LEVELS.contains(&level)
}

/// A supported level: the entry must exist and be non-nil (a null entry is
/// upstream's "not supported").
fn thinking_level_allowed(levels: &HashMap<String, Option<String>>, level: &str) -> bool {
    matches!(levels.get(level), Some(Some(_)))
}

/// The thinking level map of a model, preferring the declared models.json entry
/// over pi's cached catalog. `None` means neither source declares a map, so
/// nothing can be called unsupported.
fn thinking_levels_for(
    providers: &HashMap<String, PiProviderConfig>,
    catalog: &StartupCatalog,
    provider_id: &str,
    model_id: &str,
) -> Option<HashMap<String, Option<String>>> {
    if let Some(model) = find_provider_model(providers.get(provider_id), model_id) {
        if let Some(levels) = model.thinking_level_map.as_ref() {
            if !levels.is_empty() {
                return Some(levels.clone());
            }
        }
    }
    catalog_thinking_levels(catalog, provider_id, model_id)
}

fn find_provider_model<'a>(
    provider: Option<&'a PiProviderConfig>,
    model_id: &str,
) -> Option<&'a PiModel> {
    provider?.models.iter().find(|model| model.id == model_id)
}

pub(crate) fn sorted_provider_ids(providers: &HashMap<String, PiProviderConfig>) -> Vec<String> {
    let mut ids: Vec<String> = providers.keys().cloned().collect();
    ids.sort();
    ids
}

/// models.json's first provider id in alphabetical order (deterministic).
pub(crate) fn first_provider_id(providers: &HashMap<String, PiProviderConfig>) -> String {
    sorted_provider_ids(providers)
        .into_iter()
        .next()
        .unwrap_or_default()
}

/// The provider's first declared model (upstream keeps the list ordered, so this
/// is the model a human would pick first).
pub(crate) fn first_model_id(provider: &PiProviderConfig) -> String {
    provider
        .models
        .iter()
        .map(|model| model.id.trim().to_string())
        .find(|id| !id.is_empty())
        .unwrap_or_default()
}

fn model_ids(provider: &PiProviderConfig) -> Vec<String> {
    provider
        .models
        .iter()
        .map(|model| model.id.trim().to_string())
        .filter(|id| !id.is_empty())
        .collect()
}

fn provider_has_model(provider: &PiProviderConfig, model_id: &str) -> bool {
    provider.models.iter().any(|model| model.id == model_id)
}

/// Repair `settings.json` after models.json changed from `before` to `after`.
///
/// Returns the resulting selection with the reasons a value was replaced, and
/// only writes when something actually changed.
pub fn reconcile_startup_model_for_home(
    home_dir: &Path,
    before: &HashMap<String, PiProviderConfig>,
    after: &HashMap<String, PiProviderConfig>,
) -> Result<PiApplyResult, String> {
    let status = read_pi_settings_for_home(home_dir)?;
    let mut next = status.settings();
    let catalog = read_startup_catalog_for_home(home_dir);
    let mut reasons: Vec<String> = Vec::new();

    let mut provider = optional_value(next.default_provider.as_ref());
    let mut model = optional_value(next.default_model.as_ref());
    let mut model_replaced = false;

    // 1. The provider itself disappeared. models.json no longer supplies it and
    //    pi's catalog never did, so pi has no models under that id at all.
    if !provider.is_empty() {
        let supplied_before = before.contains_key(&provider);
        let supplied_after = after.contains_key(&provider);
        if supplied_before && !supplied_after && !knows_provider(&catalog, &provider) {
            let replacement = first_provider_id(after);
            if !replacement.is_empty() {
                let replaced_model = after
                    .get(&replacement)
                    .map(first_model_id)
                    .unwrap_or_default();
                reasons.push(format!(
                    "provider \"{provider}\" is no longer supplied by models.json; using \"{replacement}\" instead"
                ));
                provider = replacement.clone();
                next.default_provider = Some(replacement);
                model = replaced_model.clone();
                next.default_model = optional_value_ptr(&replaced_model);
                model_replaced = true;
            } else {
                reasons.push(format!(
                    "provider \"{provider}\" is no longer supplied by models.json and the profile has no providers; clearing the startup model"
                ));
                provider = String::new();
                model = String::new();
                next.default_provider = None;
                next.default_model = None;
                model_replaced = true;
            }
        }
    }

    // 2. The provider survived but the model id did not (renamed or dropped).
    if !provider.is_empty() && !model.is_empty() {
        if let Some(config) = after.get(&provider) {
            if catalog_model(&catalog, &provider, &model).is_none()
                && !provider_has_model(config, &model)
            {
                let replacement = first_model_id(config);
                if !replacement.is_empty() {
                    reasons.push(format!(
                        "model \"{model}\" is not in \"{provider}\" anymore; using \"{replacement}\" instead"
                    ));
                    model = replacement.clone();
                    next.default_model = Some(replacement);
                    model_replaced = true;
                } else {
                    reasons.push(format!(
                        "model \"{model}\" is not in \"{provider}\" anymore and the provider lists no models; clearing the startup model"
                    ));
                    model = String::new();
                    next.default_model = None;
                    model_replaced = true;
                }
            }
        }
    }

    // 3. The thinking level belongs to the model it was chosen for: drop a
    //    level the (possibly replaced) model cannot express, and never persist
    //    an unknown level (pi ignores it, so keeping it would only hide the
    //    mistake).
    let level = optional_value(next.default_thinking_level.as_ref());
    if !level.is_empty() {
        let cleared = if !is_thinking_level(&level) {
            Some(format!(
                "thinking level \"{level}\" is not one of {}; clearing it",
                THINKING_LEVELS.join(" | ")
            ))
        } else if !provider.is_empty() && !model.is_empty() {
            match thinking_levels_for(after, &catalog, &provider, &model) {
                Some(levels) if !thinking_level_allowed(&levels, &level) => Some(format!(
                    "thinking level \"{level}\" is not supported by \"{model}\"; clearing it"
                )),
                // A different model is the startup model now: the level
                // survives only when the new model declares it.
                None if model_replaced => Some(format!(
                    "thinking level \"{level}\" is not declared by \"{model}\"; clearing it"
                )),
                _ => None,
            }
        } else {
            None
        };
        if let Some(reason) = cleared {
            reasons.push(reason);
            next.default_thinking_level = None;
        }
    }

    let mut result = PiApplyResult {
        startup_adjusted: !reasons.is_empty(),
        startup_reasons: reasons,
        startup: next,
        ..Default::default()
    };
    if !result.startup_adjusted {
        return Ok(result);
    }
    save_pi_settings_for_home(home_dir, &result.startup)?;
    // Report what was actually persisted (the save normalizes empty values).
    result.startup = read_pi_settings_for_home(home_dir)?.settings();
    Ok(result)
}

/// Merge the providers models.json supplies with the cached catalog
/// (models.json wins; its provider order is alphabetical, like pi's
/// first-provider fallback).
pub fn list_startup_options_for_home(home_dir: &Path) -> Result<StartupOptions, String> {
    let config: PiCurrentConfig = crate::pi::read_pi_current_config_for_home(home_dir)?;
    let catalog = read_startup_catalog_for_home(home_dir);
    let mut options = StartupOptions::default();
    for (provider_id, entry) in &config.providers {
        options.providers.push(provider_id.clone());
        options.models.insert(provider_id.clone(), model_ids(entry));
    }
    for (provider_id, models) in &catalog {
        if options.models.contains_key(provider_id) {
            continue;
        }
        options.providers.push(provider_id.clone());
        let mut ids: Vec<String> = models.keys().cloned().collect();
        ids.sort();
        options.models.insert(provider_id.clone(), ids);
    }
    options.providers.sort();
    Ok(options)
}

/// `~/.pi/agent/settings.json` location (re-exported for the command layer).
pub fn settings_path_for_home(home_dir: &Path) -> Result<std::path::PathBuf, String> {
    pi_settings_path_for_home(home_dir)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pi::PiModel;
    use uuid::Uuid;

    fn temp_home(name: &str) -> std::path::PathBuf {
        let dir =
            std::env::temp_dir().join(format!("droidgear-pi-startup-{name}-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&dir).expect("temp home");
        dir
    }

    fn provider(_id: &str, models: &[&str]) -> PiProviderConfig {
        PiProviderConfig {
            models: models
                .iter()
                .map(|model_id| PiModel {
                    id: (*model_id).to_string(),
                    ..Default::default()
                })
                .collect(),
            ..Default::default()
        }
    }

    fn seed_settings(home: &Path, provider_id: &str, model_id: &str) {
        let path = pi_settings_path_for_home(home).expect("path");
        std::fs::create_dir_all(path.parent().expect("parent")).expect("dir");
        std::fs::write(
            &path,
            format!(
                "{{\n  \"theme\": \"dark\",\n  \"defaultProvider\": \"{provider_id}\",\n  \"defaultModel\": \"{model_id}\"\n}}\n"
            ),
        )
        .expect("seed");
    }

    #[test]
    fn reconcile_repoints_a_provider_the_apply_removed() {
        let home = temp_home("provider");
        seed_settings(&home, "gone", "m1");
        let before = HashMap::from([("gone".to_string(), provider("gone", &["m1"]))]);
        let after = HashMap::from([("kept".to_string(), provider("kept", &["m2"]))]);

        let result = reconcile_startup_model_for_home(&home, &before, &after).expect("reconcile");

        assert!(result.startup_adjusted);
        assert_eq!(result.startup.default_provider.as_deref(), Some("kept"));
        assert_eq!(result.startup.default_model.as_deref(), Some("m2"));
        assert_eq!(result.startup_reasons.len(), 1);
        // Foreign keys survive the settings rewrite.
        let text =
            std::fs::read_to_string(pi_settings_path_for_home(&home).expect("path")).expect("read");
        assert!(text.contains("\"theme\": \"dark\""));
    }

    #[test]
    fn reconcile_repoints_a_dropped_model_and_clears_its_thinking_level() {
        let home = temp_home("model");
        seed_settings(&home, "acme", "old");
        let mut settings: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(pi_settings_path_for_home(&home).expect("path"))
                .expect("read"),
        )
        .expect("json");
        settings["defaultThinkingLevel"] = serde_json::json!("high");
        std::fs::write(
            pi_settings_path_for_home(&home).expect("path"),
            settings.to_string(),
        )
        .expect("write");

        let before = HashMap::from([("acme".to_string(), provider("acme", &["old"]))]);
        let after = HashMap::from([("acme".to_string(), provider("acme", &["new"]))]);

        let result = reconcile_startup_model_for_home(&home, &before, &after).expect("reconcile");

        assert!(result.startup_adjusted);
        assert_eq!(result.startup.default_model.as_deref(), Some("new"));
        assert!(result.startup.default_thinking_level.is_none());
    }

    #[test]
    fn reconcile_leaves_catalog_only_selections_alone() {
        let home = temp_home("catalog");
        seed_settings(&home, "openrouter", "catalog-model");
        let dir = pi_config_dir_for_home(&home).expect("dir");
        std::fs::create_dir_all(&dir).expect("dir");
        std::fs::write(
            dir.join("models-store.json"),
            r#"{"openrouter":{"models":[{"id":"catalog-model"}]}}"#,
        )
        .expect("catalog");

        let before = HashMap::from([(
            "openrouter".to_string(),
            provider("openrouter", &["catalog-model"]),
        )]);
        let result =
            reconcile_startup_model_for_home(&home, &before, &HashMap::new()).expect("reconcile");

        assert!(!result.startup_adjusted);
        assert_eq!(
            result.startup.default_model.as_deref(),
            Some("catalog-model")
        );
    }

    #[test]
    fn startup_options_list_models_json_then_catalog() {
        let home = temp_home("options");
        let dir = pi_config_dir_for_home(&home).expect("dir");
        std::fs::create_dir_all(&dir).expect("dir");
        std::fs::write(
            dir.join("models.json"),
            r#"{"providers":{"acme":{"baseUrl":"https://acme.test","models":[{"id":"m2"},{"id":"m1"}]}}}"#,
        )
        .expect("models.json");
        std::fs::write(
            dir.join("models-store.json"),
            r#"{"catalog":{"models":[{"id":"z"},{"id":"a"}]}}"#,
        )
        .expect("catalog");

        let options = list_startup_options_for_home(&home).expect("options");
        assert_eq!(options.providers, vec!["acme", "catalog"]);
        assert_eq!(options.models["acme"], vec!["m2", "m1"]);
        assert_eq!(options.models["catalog"], vec!["a", "z"]);
    }
}
