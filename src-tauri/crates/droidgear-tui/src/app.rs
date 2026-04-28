use std::path::PathBuf;

use droidgear_core::{
    channel::Channel,
    codex::CodexProfile,
    factory_settings::{CustomModel, MissionModelSettings},
    hermes::HermesProfile,
    mcp::McpServer,
    openclaw::{OpenClawProfile, OpenClawSubAgent},
    opencode::OpenCodeProfile,
    paths::{EffectivePath, EffectivePaths},
    pi::PiProfile,
    sessions::SessionSummary,
    specs::SpecFile,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Main,
    Paths,
    Factory,
    FactoryModel,
    Mcp,
    McpServer,
    McpArgs,
    McpKeyValues,
    Codex,
    CodexProfile,
    CodexProvider,
    OpenCode,
    OpenCodeProfile,
    OpenCodeProvider,
    OpenCodeModel,
    OpenClaw,
    OpenClawProfile,
    OpenClawProvider,
    OpenClawModel,
    OpenClawHelpers,
    OpenClawSubagents,
    OpenClawSubagentDetail,
    Pi,
    PiProfile,
    PiProvider,
    PiModel,
    Hermes,
    HermesProfile,
    HermesProvider,
    Sessions,
    Specs,
    Channels,
    ChannelsEdit,
    Missions,
}

#[derive(Debug, Clone)]
pub struct Toast {
    pub message: String,
    pub is_error: bool,
}

#[derive(Debug, Clone)]
pub enum Modal {
    Confirm {
        message: String,
        action: ConfirmAction,
    },
    Input {
        title: String,
        value: String,
        cursor: usize,
        is_secret: bool,
        action: InputAction,
    },
    Select {
        title: String,
        options: Vec<String>,
        index: usize,
        action: SelectAction,
    },
}

#[derive(Debug, Clone)]
pub enum ConfirmAction {
    Quit,
    PathsResetKey {
        key: String,
    },
    CodexApply {
        id: String,
    },
    CodexDelete {
        id: String,
    },
    CodexDeleteProvider {
        profile_id: String,
        provider_id: String,
    },
    OpenCodeApply {
        id: String,
    },
    OpenCodeDelete {
        id: String,
    },
    OpenCodeDeleteProvider {
        profile_id: String,
        provider_id: String,
    },
    OpenCodeDeleteModel {
        profile_id: String,
        provider_id: String,
        model_id: String,
    },
    OpenClawApply {
        id: String,
    },
    OpenClawDelete {
        id: String,
    },
    OpenClawDeleteProvider {
        profile_id: String,
        provider_id: String,
    },
    OpenClawDeleteModel {
        profile_id: String,
        provider_id: String,
        model_index: usize,
    },
    McpToggle {
        name: String,
        disabled: bool,
    },
    McpDelete {
        name: String,
    },
    FactorySetDefaultModel {
        model_id: String,
    },
    FactoryDeleteModel {
        index: usize,
    },
    SessionDelete {
        path: String,
    },
    SpecDelete {
        path: String,
    },
    ChannelDelete {
        id: String,
    },
    OpenClawSubagentDelete {
        id: String,
    },
    OpenClawSubagentToggleAllow {
        id: String,
    },
    PiApply {
        id: String,
    },
    PiDelete {
        id: String,
    },
    PiDeleteProvider {
        profile_id: String,
        provider_id: String,
    },
    PiDeleteModel {
        profile_id: String,
        provider_id: String,
        model_index: usize,
    },
    HermesApply {
        id: String,
    },
    HermesDelete {
        id: String,
    },
}

#[derive(Debug, Clone)]
pub enum InputAction {
    PathsSetKey {
        key: String,
    },
    CodexCreateProfile,
    CodexDuplicate {
        id: String,
    },
    CodexSetProfileName {
        id: String,
    },
    CodexSetProfileDescription {
        id: String,
    },
    CodexSetProfileModel {
        id: String,
    },
    CodexSetProfileApiKey {
        id: String,
    },
    CodexAddProvider {
        id: String,
    },
    CodexSetProviderName {
        profile_id: String,
        provider_id: String,
    },
    CodexSetProviderBaseUrl {
        profile_id: String,
        provider_id: String,
    },
    CodexSetProviderApiKey {
        profile_id: String,
        provider_id: String,
    },
    CodexSetProviderModel {
        profile_id: String,
        provider_id: String,
    },
    OpenCodeCreateProfile,
    OpenCodeDuplicate {
        id: String,
    },
    OpenCodeSetProfileName {
        id: String,
    },
    OpenCodeSetProfileDescription {
        id: String,
    },
    OpenCodeAddProvider {
        profile_id: String,
    },
    OpenCodeSetProviderName {
        profile_id: String,
        provider_id: String,
    },
    OpenCodeSetProviderNpm {
        profile_id: String,
        provider_id: String,
    },
    OpenCodeSetProviderBaseUrl {
        profile_id: String,
        provider_id: String,
    },
    OpenCodeSetProviderApiKey {
        profile_id: String,
        provider_id: String,
    },
    OpenCodeSetProviderTimeout {
        profile_id: String,
        provider_id: String,
    },
    OpenCodeAddModel {
        profile_id: String,
        provider_id: String,
    },
    OpenCodeSetModelName {
        profile_id: String,
        provider_id: String,
        model_id: String,
    },
    OpenCodeSetModelContextLimit {
        profile_id: String,
        provider_id: String,
        model_id: String,
    },
    OpenCodeSetModelOutputLimit {
        profile_id: String,
        provider_id: String,
        model_id: String,
    },
    OpenClawCreateProfile,
    OpenClawDuplicate {
        id: String,
    },
    OpenClawSetProfileName {
        id: String,
    },
    OpenClawSetProfileDescription {
        id: String,
    },
    OpenClawAddProvider {
        profile_id: String,
    },
    OpenClawSetProviderBaseUrl {
        profile_id: String,
        provider_id: String,
    },
    OpenClawSetProviderApiKey {
        profile_id: String,
        provider_id: String,
    },
    OpenClawAddModel {
        profile_id: String,
        provider_id: String,
    },
    OpenClawSetModelId {
        profile_id: String,
        provider_id: String,
        model_index: usize,
    },
    OpenClawSetModelName {
        profile_id: String,
        provider_id: String,
        model_index: usize,
    },
    OpenClawSetModelContextWindow {
        profile_id: String,
        provider_id: String,
        model_index: usize,
    },
    OpenClawSetModelMaxTokens {
        profile_id: String,
        provider_id: String,
        model_index: usize,
    },
    OpenClawSetBlockStreamingMinChars {
        profile_id: String,
    },
    OpenClawSetBlockStreamingMaxChars {
        profile_id: String,
    },
    OpenClawSetBlockStreamingIdleMs {
        profile_id: String,
    },
    FactoryDraftSetBaseUrl,
    FactoryDraftSetApiKey,
    FactoryDraftSetModel,
    FactoryDraftSetDisplayName,
    FactoryDraftSetMaxOutputTokens,
    FactoryDraftSetExtraArgs,
    FactoryDraftSetExtraHeaders,
    McpCreateServer,
    McpDraftSetName,
    McpDraftSetCommand,
    McpDraftSetUrl,
    McpArgsAdd,
    McpArgsEdit {
        index: usize,
    },
    McpKeyValueAdd {
        mode: McpKeyValuesMode,
    },
    McpKeyValueEdit {
        mode: McpKeyValuesMode,
        index: usize,
    },
    ChannelsDraftSetName,
    ChannelsDraftSetBaseUrl,
    ChannelsDraftSetUsername,
    ChannelsDraftSetPassword,
    ChannelsDraftSetApiKey,
    OpenClawSubagentCreate,
    OpenClawSubagentSetName {
        id: String,
    },
    OpenClawSubagentSetEmoji {
        id: String,
    },
    OpenClawSubagentSetPrimaryModel {
        id: String,
    },
    OpenClawSubagentSetWorkspace {
        id: String,
    },
    PiCreateProfile,
    PiDuplicate {
        id: String,
    },
    PiSetProfileName {
        id: String,
    },
    PiSetProfileDescription {
        id: String,
    },
    PiAddProvider {
        profile_id: String,
    },
    PiSetProviderBaseUrl {
        profile_id: String,
        provider_id: String,
    },
    PiSetProviderApiKey {
        profile_id: String,
        provider_id: String,
    },
    PiAddModel {
        profile_id: String,
        provider_id: String,
    },
    PiSetModelId {
        profile_id: String,
        provider_id: String,
        model_index: usize,
    },
    PiSetModelName {
        profile_id: String,
        provider_id: String,
        model_index: usize,
    },
    PiSetModelContextWindow {
        profile_id: String,
        provider_id: String,
        model_index: usize,
    },
    PiSetModelMaxTokens {
        profile_id: String,
        provider_id: String,
        model_index: usize,
    },
    PiSetModelCost {
        profile_id: String,
        provider_id: String,
        model_index: usize,
    },
    HermesCreateProfile,
    HermesDuplicate {
        id: String,
    },
    HermesSetProfileName {
        id: String,
    },
    HermesSetProfileDescription {
        id: String,
    },
    HermesSetProfileDefaultModel {
        id: String,
    },
    HermesSetProfileProvider {
        id: String,
    },
    HermesSetProfileBaseUrl {
        id: String,
    },
    HermesSetProfileApiKey {
        id: String,
    },
    HermesImportSetApiKey {
        id: String,
    },
}

#[derive(Debug, Clone)]
pub enum SelectAction {
    GoToNav,
    CodexSetProfileModelProvider {
        id: String,
    },
    CodexSetProfileReasoningEffort {
        id: String,
    },
    CodexSetProviderWireApi {
        profile_id: String,
        provider_id: String,
    },
    CodexSetProviderReasoningEffort {
        profile_id: String,
        provider_id: String,
    },
    OpenCodeImportProviders {
        id: String,
    },
    OpenClawSetDefaultModel {
        id: String,
    },
    OpenClawAddFailoverModel {
        id: String,
    },
    OpenClawSetProviderApiType {
        profile_id: String,
        provider_id: String,
    },
    OpenClawSetBlockStreamingDefault {
        id: String,
    },
    OpenClawSetBlockStreamingBreak {
        id: String,
    },
    OpenClawSetTelegramChunkMode {
        id: String,
    },
    FactoryDraftSetProvider,
    FactoryDraftSetReasoningEffort,
    McpDraftSetType,
    ChannelsDraftSetType,
    OpenClawSubagentSetToolsProfile {
        id: String,
    },
    MissionsSetWorkerModel,
    MissionsSetWorkerReasoningEffort,
    MissionsSetValidationWorkerModel,
    MissionsSetValidationWorkerReasoningEffort,
    PiSetProviderApi {
        profile_id: String,
        provider_id: String,
    },
    HermesImportFromChannel {
        profile_id: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum McpKeyValuesMode {
    Env,
    Headers,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpenClawProfileFocus {
    Fields,
    Failover,
    Providers,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CodexDetailFocus {
    Fields,
    Providers,
}

#[derive(Debug, Clone)]
pub struct App {
    pub home_dir: PathBuf,

    pub screen: Screen,
    pub should_quit: bool,

    pub nav_index: usize,

    pub toast: Option<Toast>,
    pub modal: Option<Modal>,

    pub paths: Option<EffectivePaths>,
    pub paths_index: usize,

    pub custom_models: Vec<CustomModel>,
    pub factory_default_model_id: Option<String>,
    pub factory_models_index: usize,
    pub factory_edit_index: Option<usize>,
    pub factory_draft: Option<CustomModel>,
    pub factory_model_field_index: usize,

    pub mcp_servers: Vec<McpServer>,
    pub mcp_index: usize,
    pub mcp_edit_original_name: Option<String>,
    pub mcp_edit_draft: Option<McpServer>,
    pub mcp_edit_field_index: usize,
    pub mcp_args_index: usize,
    pub mcp_kv_mode: McpKeyValuesMode,
    pub mcp_kv_index: usize,

    pub codex_profiles: Vec<CodexProfile>,
    pub codex_active_id: Option<String>,
    pub codex_index: usize,
    pub codex_detail_id: Option<String>,
    pub codex_detail: Option<CodexProfile>,
    pub codex_detail_focus: CodexDetailFocus,
    pub codex_detail_field_index: usize,
    pub codex_detail_provider_ids: Vec<String>,
    pub codex_detail_provider_index: usize,
    pub codex_provider_id: Option<String>,
    pub codex_provider_field_index: usize,

    pub opencode_profiles: Vec<OpenCodeProfile>,
    pub opencode_active_id: Option<String>,
    pub opencode_index: usize,
    pub opencode_detail_id: Option<String>,
    pub opencode_detail: Option<OpenCodeProfile>,
    pub opencode_detail_focus: CodexDetailFocus,
    pub opencode_detail_field_index: usize,
    pub opencode_detail_provider_ids: Vec<String>,
    pub opencode_detail_provider_index: usize,
    pub opencode_provider_id: Option<String>,
    pub opencode_provider_focus: CodexDetailFocus,
    pub opencode_provider_field_index: usize,
    pub opencode_provider_model_ids: Vec<String>,
    pub opencode_provider_model_index: usize,
    pub opencode_model_id: Option<String>,
    pub opencode_model_field_index: usize,

    pub openclaw_profiles: Vec<OpenClawProfile>,
    pub openclaw_active_id: Option<String>,
    pub openclaw_index: usize,
    pub openclaw_detail_id: Option<String>,
    pub openclaw_detail: Option<OpenClawProfile>,
    pub openclaw_detail_focus: OpenClawProfileFocus,
    pub openclaw_detail_field_index: usize,
    pub openclaw_detail_failover_index: usize,
    pub openclaw_detail_provider_ids: Vec<String>,
    pub openclaw_detail_provider_index: usize,
    pub openclaw_provider_id: Option<String>,
    pub openclaw_provider_focus: CodexDetailFocus,
    pub openclaw_provider_field_index: usize,
    pub openclaw_provider_model_index: usize,
    pub openclaw_model_field_index: usize,
    pub openclaw_helpers_field_index: usize,

    pub openclaw_subagents: Vec<OpenClawSubAgent>,
    pub openclaw_subagents_index: usize,
    pub openclaw_subagent_detail: Option<OpenClawSubAgent>,
    pub openclaw_subagent_field_index: usize,

    pub pi_profiles: Vec<PiProfile>,
    pub pi_active_id: Option<String>,
    pub pi_index: usize,
    pub pi_detail_id: Option<String>,
    pub pi_detail: Option<PiProfile>,
    pub pi_detail_field_index: usize,
    pub pi_provider_index: usize,
    pub pi_provider_field_index: usize,
    pub pi_model_index: usize,
    pub pi_model_field_index: usize,

    pub hermes_profiles: Vec<HermesProfile>,
    pub hermes_active_id: Option<String>,
    pub hermes_index: usize,
    pub hermes_detail_id: Option<String>,
    pub hermes_detail: Option<HermesProfile>,
    pub hermes_detail_field_index: usize,
    pub hermes_provider_field_index: usize,
    /// Temporary state used during "import from channel" flow in TUI
    pub hermes_import_pending_base_url: Option<String>,
    pub hermes_import_pending_provider: Option<String>,

    pub sessions: Vec<SessionSummary>,
    pub sessions_index: usize,

    pub specs: Vec<SpecFile>,
    pub specs_index: usize,

    pub channels: Vec<Channel>,
    pub channels_index: usize,
    pub channels_edit_draft: Option<Channel>,
    pub channels_edit_field_index: usize,
    pub channels_edit_username: String,
    pub channels_edit_password: String,
    pub channels_edit_api_key: String,

    pub mission_settings: MissionModelSettings,
    pub mission_field_index: usize,
}

impl App {
    pub fn new(home_dir: PathBuf) -> Self {
        Self {
            home_dir,
            screen: Screen::Main,
            should_quit: false,
            nav_index: 0,
            toast: None,
            modal: None,
            paths: None,
            paths_index: 0,
            custom_models: Vec::new(),
            factory_default_model_id: None,
            factory_models_index: 0,
            factory_edit_index: None,
            factory_draft: None,
            factory_model_field_index: 0,
            mcp_servers: Vec::new(),
            mcp_index: 0,
            mcp_edit_original_name: None,
            mcp_edit_draft: None,
            mcp_edit_field_index: 0,
            mcp_args_index: 0,
            mcp_kv_mode: McpKeyValuesMode::Env,
            mcp_kv_index: 0,
            codex_profiles: Vec::new(),
            codex_active_id: None,
            codex_index: 0,
            codex_detail_id: None,
            codex_detail: None,
            codex_detail_focus: CodexDetailFocus::Fields,
            codex_detail_field_index: 0,
            codex_detail_provider_ids: Vec::new(),
            codex_detail_provider_index: 0,
            codex_provider_id: None,
            codex_provider_field_index: 0,
            opencode_profiles: Vec::new(),
            opencode_active_id: None,
            opencode_index: 0,
            opencode_detail_id: None,
            opencode_detail: None,
            opencode_detail_focus: CodexDetailFocus::Fields,
            opencode_detail_field_index: 0,
            opencode_detail_provider_ids: Vec::new(),
            opencode_detail_provider_index: 0,
            opencode_provider_id: None,
            opencode_provider_focus: CodexDetailFocus::Fields,
            opencode_provider_field_index: 0,
            opencode_provider_model_ids: Vec::new(),
            opencode_provider_model_index: 0,
            opencode_model_id: None,
            opencode_model_field_index: 0,
            openclaw_profiles: Vec::new(),
            openclaw_active_id: None,
            openclaw_index: 0,
            openclaw_detail_id: None,
            openclaw_detail: None,
            openclaw_detail_focus: OpenClawProfileFocus::Fields,
            openclaw_detail_field_index: 0,
            openclaw_detail_failover_index: 0,
            openclaw_detail_provider_ids: Vec::new(),
            openclaw_detail_provider_index: 0,
            openclaw_provider_id: None,
            openclaw_provider_focus: CodexDetailFocus::Fields,
            openclaw_provider_field_index: 0,
            openclaw_provider_model_index: 0,
            openclaw_model_field_index: 0,
            openclaw_helpers_field_index: 0,
            openclaw_subagents: Vec::new(),
            openclaw_subagents_index: 0,
            openclaw_subagent_detail: None,
            openclaw_subagent_field_index: 0,
            pi_profiles: Vec::new(),
            pi_active_id: None,
            pi_index: 0,
            pi_detail_id: None,
            pi_detail: None,
            pi_detail_field_index: 0,
            pi_provider_index: 0,
            pi_provider_field_index: 0,
            pi_model_index: 0,
            pi_model_field_index: 0,
            hermes_profiles: Vec::new(),
            hermes_active_id: None,
            hermes_index: 0,
            hermes_detail_id: None,
            hermes_detail: None,
            hermes_detail_field_index: 0,
            hermes_provider_field_index: 0,
            hermes_import_pending_base_url: None,
            hermes_import_pending_provider: None,
            sessions: Vec::new(),
            sessions_index: 0,
            specs: Vec::new(),
            specs_index: 0,
            channels: Vec::new(),
            channels_index: 0,
            channels_edit_draft: None,
            channels_edit_field_index: 0,
            channels_edit_username: String::new(),
            channels_edit_password: String::new(),
            channels_edit_api_key: String::new(),
            mission_settings: MissionModelSettings {
                worker_model: None,
                worker_reasoning_effort: None,
                validation_worker_model: None,
                validation_worker_reasoning_effort: None,
            },
            mission_field_index: 0,
        }
    }

    pub fn nav_items() -> &'static [(&'static str, Screen)] {
        &[
            ("Paths", Screen::Paths),
            ("Factory", Screen::Factory),
            ("MCP", Screen::Mcp),
            ("Codex", Screen::Codex),
            ("OpenCode", Screen::OpenCode),
            ("OpenClaw", Screen::OpenClaw),
            ("Pi", Screen::Pi),
            ("Hermes", Screen::Hermes),
            ("Sessions", Screen::Sessions),
            ("Specs", Screen::Specs),
            ("Channels", Screen::Channels),
            ("Missions", Screen::Missions),
        ]
    }

    pub fn set_toast(&mut self, message: impl Into<String>, is_error: bool) {
        self.toast = Some(Toast {
            message: message.into(),
            is_error,
        });
    }

    pub fn clear_toast(&mut self) {
        self.toast = None;
    }

    /// Get the provider ID at the current pi_provider_index.
    pub fn pi_current_provider_id(&self) -> Option<String> {
        let detail = self.pi_detail.as_ref()?;
        let mut keys: Vec<String> = detail.providers.keys().cloned().collect();
        keys.sort_by_key(|a| a.to_lowercase());
        keys.get(self.pi_provider_index).cloned()
    }

    pub fn current_paths_key(&self) -> Option<String> {
        let paths = self.paths.as_ref()?;
        let keys = [
            &paths.factory,
            &paths.opencode,
            &paths.opencode_auth,
            &paths.codex,
            &paths.openclaw,
            &paths.hermes,
        ];
        keys.get(self.paths_index).map(|p| p.key.clone())
    }

    pub fn current_paths_entry(&self) -> Option<&EffectivePath> {
        let paths = self.paths.as_ref()?;
        let entries = [
            &paths.factory,
            &paths.opencode,
            &paths.opencode_auth,
            &paths.codex,
            &paths.openclaw,
            &paths.hermes,
        ];
        entries.get(self.paths_index).copied()
    }

    pub fn clamp_indices(&mut self) {
        if self.nav_index >= Self::nav_items().len() {
            self.nav_index = Self::nav_items().len().saturating_sub(1);
        }

        let paths_count = 6;
        if self.paths_index >= paths_count {
            self.paths_index = paths_count.saturating_sub(1);
        }
        if self.factory_models_index >= self.custom_models.len() {
            self.factory_models_index = self.custom_models.len().saturating_sub(1);
        }
        let factory_model_fields_count = 10;
        if self.factory_model_field_index >= factory_model_fields_count {
            self.factory_model_field_index = factory_model_fields_count.saturating_sub(1);
        }
        if self.mcp_index >= self.mcp_servers.len() {
            self.mcp_index = self.mcp_servers.len().saturating_sub(1);
        }
        let mcp_edit_fields_count = self
            .mcp_edit_draft
            .as_ref()
            .map(|s| match s.config.server_type {
                droidgear_core::mcp::McpServerType::Stdio => 6,
                droidgear_core::mcp::McpServerType::Http => 5,
            })
            .unwrap_or(0);
        if self.mcp_edit_field_index >= mcp_edit_fields_count {
            self.mcp_edit_field_index = mcp_edit_fields_count.saturating_sub(1);
        }
        let mcp_args_count = self
            .mcp_edit_draft
            .as_ref()
            .and_then(|s| s.config.args.as_ref())
            .map(|v| v.len())
            .unwrap_or(0);
        if self.mcp_args_index >= mcp_args_count {
            self.mcp_args_index = mcp_args_count.saturating_sub(1);
        }
        let mcp_kv_count = self
            .mcp_edit_draft
            .as_ref()
            .map(|s| match self.mcp_kv_mode {
                McpKeyValuesMode::Env => s.config.env.as_ref().map(|m| m.len()).unwrap_or(0),
                McpKeyValuesMode::Headers => {
                    s.config.headers.as_ref().map(|m| m.len()).unwrap_or(0)
                }
            })
            .unwrap_or(0);
        if self.mcp_kv_index >= mcp_kv_count {
            self.mcp_kv_index = mcp_kv_count.saturating_sub(1);
        }
        if self.codex_index >= self.codex_profiles.len() {
            self.codex_index = self.codex_profiles.len().saturating_sub(1);
        }
        let codex_fields_count = 6;
        if self.codex_detail_field_index >= codex_fields_count {
            self.codex_detail_field_index = codex_fields_count.saturating_sub(1);
        }
        if self.codex_detail_provider_index >= self.codex_detail_provider_ids.len() {
            self.codex_detail_provider_index =
                self.codex_detail_provider_ids.len().saturating_sub(1);
        }
        let codex_provider_fields_count = 6;
        if self.codex_provider_field_index >= codex_provider_fields_count {
            self.codex_provider_field_index = codex_provider_fields_count.saturating_sub(1);
        }
        if self.opencode_index >= self.opencode_profiles.len() {
            self.opencode_index = self.opencode_profiles.len().saturating_sub(1);
        }
        let opencode_fields_count = 2;
        if self.opencode_detail_field_index >= opencode_fields_count {
            self.opencode_detail_field_index = opencode_fields_count.saturating_sub(1);
        }
        if self.opencode_detail_provider_index >= self.opencode_detail_provider_ids.len() {
            self.opencode_detail_provider_index =
                self.opencode_detail_provider_ids.len().saturating_sub(1);
        }
        let opencode_provider_fields_count = 5;
        if self.opencode_provider_field_index >= opencode_provider_fields_count {
            self.opencode_provider_field_index = opencode_provider_fields_count.saturating_sub(1);
        }
        if self.opencode_provider_model_index >= self.opencode_provider_model_ids.len() {
            self.opencode_provider_model_index =
                self.opencode_provider_model_ids.len().saturating_sub(1);
        }
        let opencode_model_fields_count = 3;
        if self.opencode_model_field_index >= opencode_model_fields_count {
            self.opencode_model_field_index = opencode_model_fields_count.saturating_sub(1);
        }
        if self.openclaw_index >= self.openclaw_profiles.len() {
            self.openclaw_index = self.openclaw_profiles.len().saturating_sub(1);
        }
        let openclaw_fields_count = 3;
        if self.openclaw_detail_field_index >= openclaw_fields_count {
            self.openclaw_detail_field_index = openclaw_fields_count.saturating_sub(1);
        }
        if self.openclaw_detail_failover_index
            >= self
                .openclaw_detail
                .as_ref()
                .and_then(|p| p.failover_models.as_ref())
                .map(|v| v.len())
                .unwrap_or(0)
        {
            self.openclaw_detail_failover_index = self
                .openclaw_detail
                .as_ref()
                .and_then(|p| p.failover_models.as_ref())
                .map(|v| v.len())
                .unwrap_or(0)
                .saturating_sub(1);
        }
        if self.openclaw_detail_provider_index >= self.openclaw_detail_provider_ids.len() {
            self.openclaw_detail_provider_index =
                self.openclaw_detail_provider_ids.len().saturating_sub(1);
        }
        let openclaw_provider_fields_count = 3;
        if self.openclaw_provider_field_index >= openclaw_provider_fields_count {
            self.openclaw_provider_field_index = openclaw_provider_fields_count.saturating_sub(1);
        }
        if self.openclaw_provider_model_index
            >= self
                .openclaw_detail
                .as_ref()
                .and_then(|p| {
                    self.openclaw_provider_id
                        .as_deref()
                        .and_then(|pid| p.providers.get(pid))
                })
                .map(|p| p.models.len())
                .unwrap_or(0)
        {
            self.openclaw_provider_model_index = self
                .openclaw_detail
                .as_ref()
                .and_then(|p| {
                    self.openclaw_provider_id
                        .as_deref()
                        .and_then(|pid| p.providers.get(pid))
                })
                .map(|p| p.models.len())
                .unwrap_or(0)
                .saturating_sub(1);
        }
        let openclaw_model_fields_count = 7;
        if self.openclaw_model_field_index >= openclaw_model_fields_count {
            self.openclaw_model_field_index = openclaw_model_fields_count.saturating_sub(1);
        }
        let openclaw_helpers_fields_count = 7;
        if self.openclaw_helpers_field_index >= openclaw_helpers_fields_count {
            self.openclaw_helpers_field_index = openclaw_helpers_fields_count.saturating_sub(1);
        }
        if self.openclaw_subagents_index >= self.openclaw_subagents.len() {
            self.openclaw_subagents_index = self.openclaw_subagents.len().saturating_sub(1);
        }
        let subagent_fields_count = 5;
        if self.openclaw_subagent_field_index >= subagent_fields_count {
            self.openclaw_subagent_field_index = subagent_fields_count.saturating_sub(1);
        }
        if self.pi_index >= self.pi_profiles.len() {
            self.pi_index = self.pi_profiles.len().saturating_sub(1);
        }
        // PiProfile screen: 2 fields (Name, Description) + provider list
        let pi_detail_fields_count = 2;
        let pi_provider_count = self
            .pi_detail
            .as_ref()
            .map(|p| p.providers.len())
            .unwrap_or(0);
        let pi_detail_total = pi_detail_fields_count + pi_provider_count;
        if self.pi_detail_field_index >= pi_detail_total {
            self.pi_detail_field_index = pi_detail_total.saturating_sub(1);
        }
        let pi_provider_count = self
            .pi_detail
            .as_ref()
            .map(|p| p.providers.len())
            .unwrap_or(0);
        if self.pi_provider_index >= pi_provider_count {
            self.pi_provider_index = pi_provider_count.saturating_sub(1);
        }
        // PiProvider screen has 3 fields: Base URL, API, API Key
        let pi_provider_fields_count = 3;
        if self.pi_provider_field_index >= pi_provider_fields_count {
            self.pi_provider_field_index = pi_provider_fields_count.saturating_sub(1);
        }
        let pi_model_count = self
            .pi_detail
            .as_ref()
            .and_then(|p| {
                let keys: Vec<String> = p.providers.keys().cloned().collect();
                keys.get(self.pi_provider_index)
                    .and_then(|pid| p.providers.get(pid))
                    .map(|cfg| cfg.models.len())
            })
            .unwrap_or(0);
        if self.pi_model_index >= pi_model_count {
            self.pi_model_index = pi_model_count.saturating_sub(1);
        }
        // PiModel screen has 7 fields: id, name, reasoning, input, contextWindow, maxTokens, cost
        let pi_model_fields_count = 7;
        if self.pi_model_field_index >= pi_model_fields_count {
            self.pi_model_field_index = pi_model_fields_count.saturating_sub(1);
        }
        if self.sessions_index >= self.sessions.len() {
            self.sessions_index = self.sessions.len().saturating_sub(1);
        }
        if self.specs_index >= self.specs.len() {
            self.specs_index = self.specs.len().saturating_sub(1);
        }
        if self.channels_index >= self.channels.len() {
            self.channels_index = self.channels.len().saturating_sub(1);
        }
        let channels_edit_fields_count = self
            .channels_edit_draft
            .as_ref()
            .map(|c| {
                let uses_api_key = matches!(
                    c.channel_type,
                    droidgear_core::channel::ChannelType::CliProxyApi
                        | droidgear_core::channel::ChannelType::Ollama
                        | droidgear_core::channel::ChannelType::General
                );
                if uses_api_key {
                    5
                } else {
                    6
                }
            })
            .unwrap_or(0);
        if self.channels_edit_field_index >= channels_edit_fields_count {
            self.channels_edit_field_index = channels_edit_fields_count.saturating_sub(1);
        }
        let mission_fields_count = 4;
        if self.mission_field_index >= mission_fields_count {
            self.mission_field_index = mission_fields_count.saturating_sub(1);
        }
        if self.hermes_index >= self.hermes_profiles.len() {
            self.hermes_index = self.hermes_profiles.len().saturating_sub(1);
        }
        // HermesProfile screen has 6 fields: Name, Description, Default Model, Provider, Base URL, API Key
        let hermes_detail_fields_count = 6;
        if self.hermes_detail_field_index >= hermes_detail_fields_count {
            self.hermes_detail_field_index = hermes_detail_fields_count.saturating_sub(1);
        }
        // HermesProvider screen has 4 fields: Default Model, Provider, Base URL, API Key
        let hermes_provider_fields_count = 4;
        if self.hermes_provider_field_index >= hermes_provider_fields_count {
            self.hermes_provider_field_index = hermes_provider_fields_count.saturating_sub(1);
        }
    }
}
