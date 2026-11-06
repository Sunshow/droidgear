//! OMP model roles vs models.yml: keeping role assignments resolvable.
//!
//! config.yml's `modelRoles` maps every role name to `provider/model-id` (OMP's
//! own `--model` syntax). Deleting a custom provider — or shrinking its model
//! list — in models.yml leaves those references dangling, and OMP does not
//! recover from both cases the same way:
//!
//! - provider gone: `No default model selected. …` (OMP refuses to start)
//! - model gone: `Warning: Default model proxy/m2 not found. Using proxy/m1`
//!
//! [`reconcile_model_roles_for_home`] is the single repair path, the OMP twin of
//! Pi's `ReconcileStartupModel`. It only rewrites a role that the models.yml
//! write made unresolvable, and it never touches a role pointing at a provider
//! OMP's own model cache (models.db `model_cache`) knows: those are catalog
//! providers that models.yml was merely overriding, so removing the override
//! does not remove the provider.
//!
//! Every repair lands on the first available `provider/model-id`: models.yml
//! providers first (alphabetical id, declaration order for models — the order a
//! human reads the file in), then the cached catalog. With nothing left to point
//! at, the role is written as `null`, which is exactly how OMP records an
//! unassigned role.

use serde::{Deserialize, Serialize};
use specta::Type;
use std::collections::HashMap;
use std::path::Path;

use crate::omp::{
    read_model_cache_for_home, read_omp_models_yml_value_for_home, OmpAgentConfig, OmpModelRoles,
    OmpThinkingConfig,
};

/// OMP's "let the model decide" level (`--thinking` accepts `auto` alongside
/// `off…max`), and the value config.yml falls back to.
pub const OMP_AUTO_THINKING: &str = "auto";

/// OMP's `--thinking` enum (see `omp --help`).
pub const OMP_THINKING_LEVELS: [&str; 8] = [
    "off",
    "minimal",
    "low",
    "medium",
    "high",
    "xhigh",
    "max",
    OMP_AUTO_THINKING,
];

/// The upstream role set in display order (chat roles first, then the
/// model-kind roles). Unknown roles live in [`OmpModelRoles::extra`].
pub const OMP_ROLE_NAMES: [&str; 15] = [
    "default",
    "smol",
    "slow",
    "vision",
    "plan",
    "commit",
    "tiny",
    "memory",
    "task",
    "advisor",
    "image",
    "web",
    "speech",
    "dictation",
    "judge",
];

/// One role a models.yml write made unresolvable, plus where it moved.
/// `to == None` means the role was cleared.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct OmpRoleAdjustment {
    pub role: String,
    pub from: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to: Option<String>,
    pub reason: String,
}

/// A `defaultThinkingLevel` reset: the model the default role now points at
/// cannot express the level, so config.yml falls back to `auto` instead of
/// silently sending an unsupported effort.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct OmpThinkingAdjustment {
    pub from: String,
    pub to: String,
    pub reason: String,
}

/// The repair pass. `adjusted` is true when config.yml was rewritten.
#[derive(Debug, Clone, Default, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct OmpRoleReconcile {
    pub adjusted: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub adjustments: Vec<OmpRoleAdjustment>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking: Option<OmpThinkingAdjustment>,
    pub roles: OmpModelRoles,
}

/// The list of values a role input can pick from: models.yml merged with the
/// cached catalog, provider ids alphabetical, models.yml model order preserved.
#[derive(Debug, Clone, Default, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct OmpRoleOptions {
    pub providers: Vec<String>,
    pub models: HashMap<String, Vec<String>>,
}

/// Read a modelled role.
pub fn role_value(roles: &OmpModelRoles, name: &str) -> Option<String> {
    let value = match name {
        "default" => roles.default.clone(),
        "smol" => roles.smol.clone(),
        "slow" => roles.slow.clone(),
        "vision" => roles.vision.clone(),
        "plan" => roles.plan.clone(),
        "commit" => roles.commit.clone(),
        "tiny" => roles.tiny.clone(),
        "memory" => roles.memory.clone(),
        "task" => roles.task.clone(),
        "advisor" => roles.advisor.clone(),
        "image" => roles.image.clone(),
        "web" => roles.web.clone(),
        "speech" => roles.speech.clone(),
        "dictation" => roles.dictation.clone(),
        "judge" => roles.judge.clone(),
        _ => return None,
    }?;
    if value.trim().is_empty() {
        None
    } else {
        Some(value)
    }
}

/// Write a modelled role (`None` clears it).
pub fn set_role_value(roles: &mut OmpModelRoles, name: &str, value: Option<String>) {
    let value = value.filter(|v| !v.trim().is_empty());
    match name {
        "default" => roles.default = value,
        "smol" => roles.smol = value,
        "slow" => roles.slow = value,
        "vision" => roles.vision = value,
        "plan" => roles.plan = value,
        "commit" => roles.commit = value,
        "tiny" => roles.tiny = value,
        "memory" => roles.memory = value,
        "task" => roles.task = value,
        "advisor" => roles.advisor = value,
        "image" => roles.image = value,
        "web" => roles.web = value,
        "speech" => roles.speech = value,
        "dictation" => roles.dictation = value,
        "judge" => roles.judge = value,
        _ => {}
    }
}

/// Unknown role names stored in `extra` (upstream additions).
fn extra_role_names(roles: &OmpModelRoles) -> Vec<String> {
    let mut names: Vec<String> = roles
        .extra
        .keys()
        .filter(|name| !OMP_ROLE_NAMES.contains(&name.as_str()))
        .cloned()
        .collect();
    names.sort();
    names
}

fn extra_role_value(roles: &OmpModelRoles, name: &str) -> Option<String> {
    let value = roles.extra.get(name)?.as_str()?.trim().to_string();
    if value.is_empty() {
        None
    } else {
        Some(value)
    }
}

/// OMP's cached provider catalog: provider id → model id → cached thinking
/// metadata (`None` when the cache entry listed none).
type RoleCatalog = HashMap<String, HashMap<String, Option<OmpThinkingConfig>>>;

fn read_role_catalog_for_home(home_dir: &Path) -> Option<RoleCatalog> {
    let entries = read_model_cache_for_home(home_dir).ok()?;
    let mut catalog: RoleCatalog = HashMap::new();
    for entry in entries {
        if entry.provider_id.trim().is_empty() {
            continue;
        }
        let models = catalog.entry(entry.provider_id).or_default();
        for model in entry.models {
            if !model.id.trim().is_empty() {
                models.insert(model.id, model.thinking);
            }
        }
    }
    if catalog.is_empty() {
        None
    } else {
        Some(catalog)
    }
}

/// The untyped models.yml provider map (what the reconcile compares).
fn read_provider_map_for_home(
    home_dir: &Path,
) -> Result<HashMap<String, serde_json::Value>, String> {
    read_omp_models_yml_value_for_home(home_dir)
}

fn provider_model_ids(provider: &serde_json::Value) -> Vec<String> {
    let Some(models) = provider.get("models").and_then(|value| value.as_array()) else {
        return Vec::new();
    };
    models
        .iter()
        .filter_map(|model| model.get("id").and_then(|value| value.as_str()))
        .map(|id| id.trim().to_string())
        .filter(|id| !id.is_empty())
        .collect()
}

fn provider_model_thinking(
    provider: &serde_json::Value,
    model_id: &str,
) -> Option<Option<OmpThinkingConfig>> {
    let models = provider.get("models")?.as_array()?;
    for model in models {
        if model.get("id").and_then(|value| value.as_str()) != Some(model_id) {
            continue;
        }
        let Some(raw) = model.get("thinking") else {
            return Some(None);
        };
        if raw.is_null() {
            return Some(None);
        }
        return Some(serde_json::from_value(raw.clone()).ok());
    }
    None
}

/// What a role may point at once models.yml has been written.
struct RoleTargets {
    /// The models.yml provider maps around the write. `after` is authoritative
    /// for what exists now; `before` proves this write removed the provider (an
    /// unrelated stale role is left untouched).
    before: HashMap<String, serde_json::Value>,
    after: HashMap<String, serde_json::Value>,
    /// OMP's cached provider catalog. A provider listed there survives the
    /// removal of a models.yml override, so its roles stay valid; cached models
    /// also carry the thinking metadata the level check needs.
    catalog: Option<RoleCatalog>,
}

impl RoleTargets {
    fn new(
        before: HashMap<String, serde_json::Value>,
        after: HashMap<String, serde_json::Value>,
        catalog: Option<RoleCatalog>,
    ) -> Self {
        Self {
            before,
            after,
            catalog,
        }
    }

    fn catalog_knows_provider(&self, provider_id: &str) -> bool {
        self.catalog
            .as_ref()
            .is_some_and(|catalog| catalog.contains_key(provider_id))
    }

    fn knows_model(&self, provider_id: &str, model_id: &str) -> bool {
        if provider_model_ids(&self.after[provider_id])
            .iter()
            .any(|id| id == model_id)
        {
            return true;
        }
        self.catalog
            .as_ref()
            .and_then(|catalog| catalog.get(provider_id))
            .is_some_and(|models| models.contains_key(model_id))
    }

    fn first_model_of(&self, provider_id: &str) -> String {
        self.after
            .get(provider_id)
            .and_then(|provider| provider_model_ids(provider).into_iter().next())
            .unwrap_or_default()
    }

    /// models.yml's first model across providers (alphabetical provider id),
    /// then the cached catalog.
    fn first_candidate(&self) -> String {
        let mut ids: Vec<&String> = self.after.keys().collect();
        ids.sort();
        for id in ids {
            let model = self.first_model_of(id);
            if !model.is_empty() {
                return format!("{id}/{model}");
            }
        }
        let Some(catalog) = self.catalog.as_ref() else {
            return String::new();
        };
        let mut catalog_ids: Vec<&String> = catalog.keys().collect();
        catalog_ids.sort();
        for id in catalog_ids {
            let mut models: Vec<&String> = catalog[id].keys().collect();
            models.sort();
            if let Some(model) = models.first() {
                return format!("{id}/{model}");
            }
        }
        String::new()
    }

    /// The replacement value and a reason; `None` reason means the role is fine.
    fn repair(&self, value: &str) -> (Option<String>, Option<String>) {
        let Some((provider, model)) = split_role_target(value) else {
            // A bare model id carries no provider, so a models.yml write cannot
            // be blamed for it.
            return (None, None);
        };
        if self.after.contains_key(&provider) {
            if model.is_empty() || self.knows_model(&provider, &model) {
                return (None, None);
            }
            let reason = format!("model \"{model}\" is not in provider \"{provider}\" anymore");
            let replacement = self.first_model_of(&provider);
            if !replacement.is_empty() {
                return (Some(format!("{provider}/{replacement}")), Some(reason));
            }
            let candidate = self.first_candidate();
            if !candidate.is_empty() {
                return (Some(candidate), Some(reason));
            }
            return (
                None,
                Some(format!("{reason} and no provider/model is left")),
            );
        }
        if !self.before.contains_key(&provider) {
            // The provider was never supplied by models.yml, so this write did
            // not remove it; a stale value here is a profile/typo problem that
            // `validate_model_roles` reports instead.
            return (None, None);
        }
        if self.catalog.is_none() || self.catalog_knows_provider(&provider) {
            return (None, None);
        }
        let reason = format!("provider \"{provider}\" is no longer in models.yml");
        let candidate = self.first_candidate();
        if !candidate.is_empty() {
            (Some(candidate), Some(reason))
        } else {
            (
                None,
                Some(format!("{reason} and no provider/model is left")),
            )
        }
    }

    /// The thinking metadata for a model, preferring models.yml (declared state)
    /// over the cached catalog. `None` means the model is unknown.
    fn model_thinking(
        &self,
        provider_id: &str,
        model_id: &str,
    ) -> Option<Option<OmpThinkingConfig>> {
        if let Some(provider) = self.after.get(provider_id) {
            if let Some(thinking) = provider_model_thinking(provider, model_id) {
                return Some(thinking);
            }
            if provider_model_ids(provider).iter().any(|id| id == model_id) {
                return Some(None);
            }
        }
        self.catalog
            .as_ref()
            .and_then(|catalog| catalog.get(provider_id))
            .and_then(|models| models.get(model_id).cloned())
    }

    /// Reset `defaultThinkingLevel` to `auto` when the model the default role
    /// points at cannot express it.
    ///
    /// `strict` means this pass replaced the default role, so the level was
    /// chosen for a different model and must not survive unless the new model
    /// declares it; without `strict` only a model that explicitly lists its
    /// efforts can invalidate the level.
    fn reconcile_thinking(
        &self,
        level: Option<&str>,
        default_role: Option<&str>,
        strict: bool,
    ) -> Option<OmpThinkingAdjustment> {
        let current = level?.trim();
        if current.is_empty() || current == OMP_AUTO_THINKING {
            return None;
        }
        let reset = |reason: String| {
            Some(OmpThinkingAdjustment {
                from: current.to_string(),
                to: OMP_AUTO_THINKING.to_string(),
                reason,
            })
        };
        if !is_omp_thinking_level(current) {
            return reset(format!(
                "thinking level \"{current}\" is not one of {}",
                OMP_THINKING_LEVELS.join(" | ")
            ));
        }
        let target = split_role_target(default_role.unwrap_or(""));
        let Some((provider, model)) = target else {
            return if strict {
                reset("the default role no longer points at a provider/model".to_string())
            } else {
                None
            };
        };
        let thinking = self.model_thinking(&provider, &model);
        let Some(thinking) = thinking else {
            return if strict {
                reset(format!(
                    "provider \"{provider}\" does not list model \"{model}\" anymore"
                ))
            } else {
                None
            };
        };
        let Some(thinking) = thinking else {
            return if strict {
                reset(format!("\"{model}\" does not declare any thinking levels"))
            } else {
                None
            };
        };
        if thinking_mode_disables(&thinking) {
            return reset(format!("\"{model}\" has thinking disabled"));
        }
        let efforts = thinking_efforts(&thinking);
        if efforts.is_empty() {
            return if strict {
                reset(format!("\"{model}\" does not declare its thinking levels"))
            } else {
                None
            };
        }
        if efforts.iter().any(|effort| effort == current) {
            return None;
        }
        reset(format!("\"{model}\" supports only {}", efforts.join(" | ")))
    }
}

/// Split OMP's `provider/model-id` role value at the first slash: provider ids
/// never contain one, while model ids can (e.g. `groq/compound`).
pub fn split_role_target(value: &str) -> Option<(String, String)> {
    let value = value.trim();
    let index = value.find('/')?;
    if index == 0 || index + 1 >= value.len() {
        return None;
    }
    Some((value[..index].to_string(), value[index + 1..].to_string()))
}

fn is_omp_thinking_level(level: &str) -> bool {
    OMP_THINKING_LEVELS.contains(&level)
}

fn thinking_efforts(thinking: &OmpThinkingConfig) -> Vec<String> {
    thinking
        .efforts
        .as_ref()
        .map(|efforts| {
            efforts
                .iter()
                .map(|effort| effort.trim().to_string())
                .filter(|effort| !effort.is_empty())
                .collect()
        })
        .unwrap_or_default()
}

fn thinking_mode_disables(thinking: &OmpThinkingConfig) -> bool {
    matches!(
        thinking.mode.as_deref().map(str::trim),
        Some("none" | "off" | "disabled")
    )
}

/// Repair config.yml's roles after models.yml changed from `before` to `after`
/// (both are models.yml provider maps).
pub fn reconcile_model_roles_for_home(
    home_dir: &Path,
    before: &HashMap<String, serde_json::Value>,
    after: &HashMap<String, serde_json::Value>,
) -> Result<OmpRoleReconcile, String> {
    let mut config: OmpAgentConfig = crate::omp::read_omp_agent_config_for_home(home_dir)?;
    let mut roles = config.model_roles.clone().unwrap_or_default();
    let targets = RoleTargets::new(
        before.clone(),
        after.clone(),
        read_role_catalog_for_home(home_dir),
    );

    let mut result = OmpRoleReconcile {
        roles: roles.clone(),
        ..Default::default()
    };
    let mut changed = false;
    let mut default_role_replaced = false;

    for name in OMP_ROLE_NAMES {
        let Some(from) = role_value(&roles, name) else {
            continue;
        };
        let (to, reason) = targets.repair(&from);
        let Some(reason) = reason else {
            continue;
        };
        set_role_value(&mut roles, name, to.clone());
        changed = true;
        if name == "default" {
            default_role_replaced = true;
        }
        result.adjustments.push(OmpRoleAdjustment {
            role: name.to_string(),
            from,
            to,
            reason,
        });
    }

    // Roles this build does not model (upstream additions) are string values in
    // `extra`; repair them the same way instead of leaving a stale reference.
    for name in extra_role_names(&roles) {
        let Some(from) = extra_role_value(&roles, &name) else {
            continue;
        };
        let (to, reason) = targets.repair(&from);
        let Some(reason) = reason else {
            continue;
        };
        match to.as_deref() {
            Some(value) => {
                roles
                    .extra
                    .insert(name.clone(), serde_json::Value::String(value.to_string()));
            }
            None => {
                roles.extra.insert(name.clone(), serde_json::Value::Null);
            }
        }
        changed = true;
        result.adjustments.push(OmpRoleAdjustment {
            role: name,
            from,
            to,
            reason,
        });
    }

    // The thinking level belongs to the model the default role now points at: a
    // replaced default role is strict (the old level was chosen for the old
    // model); otherwise only an explicit exclusion resets it.
    if let Some(adjustment) = targets.reconcile_thinking(
        config.default_thinking_level.as_deref(),
        role_value(&roles, "default").as_deref(),
        default_role_replaced,
    ) {
        config.default_thinking_level = Some(OMP_AUTO_THINKING.to_string());
        result.thinking = Some(adjustment);
        changed = true;
    }

    if !changed {
        return Ok(result);
    }
    config.model_roles = Some(roles.clone());
    crate::omp::write_omp_agent_config_for_home(home_dir, &config)?;
    result.roles = roles;
    result.adjusted = true;
    Ok(result)
}

/// Merge models.yml with OMP's cached catalog so the GUI pickers and the TUI
/// hints offer the same choices. models.yml wins for a provider both sources
/// know (it is the declared state).
pub fn list_omp_role_options_for_home(home_dir: &Path) -> Result<OmpRoleOptions, String> {
    let providers = read_provider_map_for_home(home_dir)?;
    let mut options = OmpRoleOptions::default();
    for (provider_id, provider) in &providers {
        options.providers.push(provider_id.clone());
        options
            .models
            .insert(provider_id.clone(), provider_model_ids(provider));
    }
    options.providers.sort();
    let Some(catalog) = read_role_catalog_for_home(home_dir) else {
        return Ok(options);
    };
    for (provider_id, models) in &catalog {
        if options.models.contains_key(provider_id) {
            continue;
        }
        let mut ids: Vec<String> = models.keys().cloned().collect();
        ids.sort();
        options.providers.push(provider_id.clone());
        options.models.insert(provider_id.clone(), ids);
    }
    options.providers.sort();
    Ok(options)
}

/// Flatten [`OmpRoleOptions`] into `provider/model-id` strings — the value a
/// role input holds.
pub fn role_values(options: &OmpRoleOptions) -> Vec<String> {
    let mut values = Vec::new();
    for provider_id in &options.providers {
        if let Some(models) = options.models.get(provider_id) {
            for model_id in models {
                values.push(format!("{provider_id}/{model_id}"));
            }
        }
    }
    values
}

/// Explain every role OMP cannot resolve. It never rewrites anything: applying a
/// profile reports the notes next to its receipt so a profile that pins a
/// deleted provider fails loudly instead of at the next OMP start.
pub fn validate_model_roles_for_home(
    home_dir: &Path,
    roles: &OmpModelRoles,
    thinking_level: Option<&str>,
) -> Result<Vec<String>, String> {
    let providers = read_provider_map_for_home(home_dir)?;
    let catalog = read_role_catalog_for_home(home_dir);
    let targets = RoleTargets::new(providers.clone(), providers, catalog);

    let mut warnings = Vec::new();
    let mut check = |name: &str, value: &str| {
        let Some((provider, model)) = split_role_target(value) else {
            return;
        };
        if targets.after.contains_key(&provider) {
            if !model.is_empty() && !targets.knows_model(&provider, &model) {
                warnings.push(format!(
                    "modelRoles.{name} points at \"{value}\"; provider \"{provider}\" does not list that model (OMP falls back to \"{}\")",
                    targets.first_model_of(&provider)
                ));
            }
            return;
        }
        if targets.catalog_knows_provider(&provider) {
            return;
        }
        warnings.push(format!(
            "modelRoles.{name} points at \"{value}\"; provider \"{provider}\" is not in models.yml or OMP's catalog (OMP exits with \"No default model selected\")"
        ));
    };

    for name in OMP_ROLE_NAMES {
        if let Some(value) = role_value(roles, name) {
            check(name, value.trim());
        }
    }
    for name in extra_role_names(roles) {
        if let Some(value) = extra_role_value(roles, &name) {
            check(&name, value.trim());
        }
    }
    if let Some(adjustment) = targets.reconcile_thinking(
        thinking_level,
        role_value(roles, "default").as_deref(),
        false,
    ) {
        warnings.push(format!(
            "defaultThinkingLevel \"{}\" cannot be expressed by the default model ({}); OMP falls back to auto",
            adjustment.from, adjustment.reason
        ));
    }
    Ok(warnings)
}
