//! Model mapping before forwarding upstream.

use crate::{app_config::AppType, provider::Provider};
use serde_json::Value;

pub struct ModelMapping {
    pub haiku_model: Option<String>,
    pub sonnet_model: Option<String>,
    pub opus_model: Option<String>,
    pub default_model: Option<String>,
    pub reasoning_model: Option<String>,
    /// Codex-only forced model from provider settings_config.config (TOML: model = "...").
    pub force_model: Option<String>,
}

impl ModelMapping {
    pub fn from_provider(provider: &Provider, app_type: &AppType) -> Self {
        let env = provider.settings_config.get("env");

        let force_model = if matches!(app_type, AppType::Codex) {
            extract_codex_model_from_provider(provider)
        } else {
            None
        };

        Self {
            haiku_model: env
                .and_then(|e| e.get("ANTHROPIC_DEFAULT_HAIKU_MODEL"))
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .map(String::from),
            sonnet_model: env
                .and_then(|e| e.get("ANTHROPIC_DEFAULT_SONNET_MODEL"))
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .map(String::from),
            opus_model: env
                .and_then(|e| e.get("ANTHROPIC_DEFAULT_OPUS_MODEL"))
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .map(String::from),
            default_model: env
                .and_then(|e| e.get("ANTHROPIC_MODEL"))
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .map(String::from),
            reasoning_model: env
                .and_then(|e| e.get("ANTHROPIC_REASONING_MODEL"))
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .map(String::from),
            force_model,
        }
    }

    pub fn has_mapping(&self) -> bool {
        self.force_model.is_some()
            || self.haiku_model.is_some()
            || self.sonnet_model.is_some()
            || self.opus_model.is_some()
            || self.default_model.is_some()
            || self.reasoning_model.is_some()
    }

    pub fn map_model(&self, original_model: &str, has_thinking: bool) -> String {
        let model_lower = original_model.to_lowercase();

        if has_thinking {
            if let Some(ref m) = self.reasoning_model {
                return m.clone();
            }
        }

        if model_lower.contains("haiku") {
            if let Some(ref m) = self.haiku_model {
                return m.clone();
            }
        }

        if model_lower.contains("opus") {
            if let Some(ref m) = self.opus_model {
                return m.clone();
            }
        }

        if model_lower.contains("sonnet") {
            if let Some(ref m) = self.sonnet_model {
                return m.clone();
            }
        }

        if let Some(ref m) = self.default_model {
            return m.clone();
        }

        original_model.to_string()
    }
}

fn extract_codex_model_from_provider(provider: &Provider) -> Option<String> {
    let config_text = provider
        .settings_config
        .get("config")
        .and_then(|v| v.as_str())?
        .trim();

    if config_text.is_empty() {
        return None;
    }

    let doc = config_text.parse::<toml_edit::DocumentMut>().ok()?;
    doc.get("model")
        .and_then(|item| item.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(String::from)
}

pub fn has_thinking_enabled(body: &Value) -> bool {
    match body
        .get("thinking")
        .and_then(|v| v.as_object())
        .and_then(|o| o.get("type"))
        .and_then(|t| t.as_str())
    {
        Some("enabled") | Some("adaptive") => true,
        Some("disabled") | None => false,
        Some(other) => {
            log::warn!("[ModelMapper] unknown thinking.type='{other}', fallback disabled");
            false
        }
    }
}

/// Returns: (mapped_body, original_model, mapped_model)
pub fn apply_model_mapping(
    mut body: Value,
    provider: &Provider,
    app_type: &AppType,
) -> (Value, Option<String>, Option<String>) {
    let mapping = ModelMapping::from_provider(provider, app_type);
    let original_model = body.get("model").and_then(|m| m.as_str()).map(String::from);

    // Codex: always force to provider-configured model when configured.
    if matches!(app_type, AppType::Codex) {
        if let Some(force_model) = mapping.force_model.clone() {
            let changed = original_model
                .as_ref()
                .map(|m| m != &force_model)
                .unwrap_or(true);

            if changed {
                log::debug!(
                    "[ModelMapper] Codex force mapping: {} -> {}",
                    original_model.as_deref().unwrap_or("<none>"),
                    force_model
                );
            }

            body["model"] = serde_json::json!(force_model.clone());
            return (body, original_model, Some(force_model));
        }

        return (body, original_model, None);
    }

    if !mapping.has_mapping() {
        return (body, original_model, None);
    }

    if let Some(ref original) = original_model {
        let has_thinking = has_thinking_enabled(&body);
        let mapped = mapping.map_model(original, has_thinking);

        if mapped != *original {
            log::debug!("[ModelMapper] model mapping: {original} -> {mapped}");
            body["model"] = serde_json::json!(mapped.clone());
            return (body, Some(original.clone()), Some(mapped));
        }
    }

    (body, original_model, None)
}
