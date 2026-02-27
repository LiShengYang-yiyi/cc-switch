//! 妯″瀷鏄犲皠妯″潡
//!
//! 鍦ㄨ姹傝浆鍙戝墠锛屾牴鎹?Provider 閰嶇疆鏇挎崲璇锋眰涓殑妯″瀷鍚嶇О

use crate::{app_config::AppType, provider::Provider};
use serde_json::Value;

/// 妯″瀷鏄犲皠閰嶇疆
pub struct ModelMapping {
    pub haiku_model: Option<String>,
    pub sonnet_model: Option<String>,
    pub opus_model: Option<String>,
    pub default_model: Option<String>,
    pub reasoning_model: Option<String>,
    pub force_model: Option<String>,
}

impl ModelMapping {
    /// 浠?Provider 閰嶇疆涓彁鍙栨ā鍨嬫槧灏?    pub fn from_provider(provider: &Provider, app_type: &AppType) -> Self {
        let env = provider.settings_config.get("env");

        // Codex: 榛樿寮哄埗浣跨敤 provider 閰嶇疆涓殑 model锛堟潵鑷?settings_config.config TOML锛?        let force_model = if matches!(app_type, AppType::Codex) {
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

    /// 妫€鏌ユ槸鍚﹂厤缃簡浠讳綍妯″瀷鏄犲皠
    pub fn has_mapping(&self) -> bool {
        self.force_model.is_some()
            || self.haiku_model.is_some()
            || self.sonnet_model.is_some()
            || self.opus_model.is_some()
            || self.default_model.is_some()
            || self.reasoning_model.is_some()
    }

    /// 鏍规嵁鍘熷妯″瀷鍚嶇О鑾峰彇鏄犲皠鍚庣殑妯″瀷
    pub fn map_model(&self, original_model: &str, has_thinking: bool) -> String {
        let model_lower = original_model.to_lowercase();

        // 1. thinking 妯″紡浼樺厛浣跨敤鎺ㄧ悊妯″瀷
        if has_thinking {
            if let Some(ref m) = self.reasoning_model {
                return m.clone();
            }
        }

        // 2. 鎸夋ā鍨嬬被鍨嬪尮閰?        if model_lower.contains("haiku") {
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

        // 3. 榛樿妯″瀷
        if let Some(ref m) = self.default_model {
            return m.clone();
        }

        // 4. 鏃犳槧灏勶紝淇濇寔鍘熸牱
        original_model.to_string()
    }
}

/// 浠?Codex provider 閰嶇疆涓彁鍙?model锛坰ettings_config.config 鏄?TOML 瀛楃涓诧級
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

/// 妫€娴嬭姹傛槸鍚﹀惎鐢ㄤ簡 thinking 妯″紡
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
            log::warn!(
                "[ModelMapper] 鏈煡 thinking.type='{other}'锛屾寜 disabled 澶勭悊浠ラ伩鍏嶈璺敱 reasoning 妯″瀷"
            );
            false
        }
    }
}

/// 瀵硅姹備綋搴旂敤妯″瀷鏄犲皠
///
/// 杩斿洖 (鏄犲皠鍚庣殑璇锋眰浣? 鍘熷妯″瀷鍚? 鏄犲皠鍚庢ā鍨嬪悕)
pub fn apply_model_mapping(
    mut body: Value,
    provider: &Provider,
    app_type: &AppType,
) -> (Value, Option<String>, Option<String>) {
    let mapping = ModelMapping::from_provider(provider, app_type);

    // 鎻愬彇鍘熷妯″瀷鍚?    let original_model = body.get("model").and_then(|m| m.as_str()).map(String::from);

    // Codex: 榛樿寮哄埗鏀瑰啓涓?provider 閰嶇疆妯″瀷
    if matches!(app_type, AppType::Codex) {
        if let Some(force_model) = mapping.force_model.clone() {
            let need_update = original_model
                .as_ref()
                .map(|m| m != &force_model)
                .unwrap_or(true);

            if need_update {
                log::debug!(
                    "[ModelMapper] Codex 寮哄埗妯″瀷鏄犲皠: {} 鈫?{}",
                    original_model.as_deref().unwrap_or("<none>"),
                    force_model
                );
            }

            body["model"] = serde_json::json!(force_model.clone());
            return (body, original_model, Some(force_model));
        }

        // 鏃犲彲鐢ㄥ己鍒舵ā鍨嬮厤缃細淇濇寔鍘熸牱
        return (body, original_model, None);
    }

    // 闈?Codex锛氱淮鎸佸師鏈夋槧灏勯€昏緫
    if !mapping.has_mapping() {
        return (body, original_model, None);
    }

    if let Some(ref original) = original_model {
        let has_thinking = has_thinking_enabled(&body);
        let mapped = mapping.map_model(original, has_thinking);

        if mapped != *original {
            log::debug!("[ModelMapper] 妯″瀷鏄犲皠: {original} 鈫?{mapped}");
            body["model"] = serde_json::json!(mapped);
            return (body, Some(original.clone()), Some(mapped));
        }
    }

    (body, original_model, None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app_config::AppType;
    use serde_json::json;

    fn create_provider_with_mapping() -> Provider {
        Provider {
            id: "test".to_string(),
            name: "Test".to_string(),
            settings_config: json!({
                "env": {
                    "ANTHROPIC_MODEL": "default-model",
                    "ANTHROPIC_DEFAULT_HAIKU_MODEL": "haiku-mapped",
                    "ANTHROPIC_DEFAULT_SONNET_MODEL": "sonnet-mapped",
                    "ANTHROPIC_DEFAULT_OPUS_MODEL": "opus-mapped",
                    "ANTHROPIC_REASONING_MODEL": "reasoning-model"
                }
            }),
            website_url: None,
            category: None,
            created_at: None,
            sort_index: None,
            notes: None,
            meta: None,
            icon: None,
            icon_color: None,
            in_failover_queue: false,
        }
    }

    fn create_provider_without_mapping() -> Provider {
        Provider {
            id: "test".to_string(),
            name: "Test".to_string(),
            settings_config: json!({}),
            website_url: None,
            category: None,
            created_at: None,
            sort_index: None,
            notes: None,
            meta: None,
            icon: None,
            icon_color: None,
            in_failover_queue: false,
        }
    }

    fn create_codex_provider_with_model(model: &str) -> Provider {
        Provider {
            id: "codex-test".to_string(),
            name: "CodexTest".to_string(),
            settings_config: json!({
                "config": format!("model = \"{}\"\n", model)
            }),
            website_url: None,
            category: None,
            created_at: None,
            sort_index: None,
            notes: None,
            meta: None,
            icon: None,
            icon_color: None,
            in_failover_queue: false,
        }
    }

    fn create_codex_provider_without_model() -> Provider {
        Provider {
            id: "codex-test".to_string(),
            name: "CodexTest".to_string(),
            settings_config: json!({
                "config": "base_url = \"https://api.openai.com/v1\"\n"
            }),
            website_url: None,
            category: None,
            created_at: None,
            sort_index: None,
            notes: None,
            meta: None,
            icon: None,
            icon_color: None,
            in_failover_queue: false,
        }
    }

    #[test]
    fn test_codex_force_model_mapping_with_any_input_model() {
        let provider = create_codex_provider_with_model("gpt-5.3-codex-high");
        let body = json!({"model": "anything-client-sent"});
        let (result, original, mapped) = apply_model_mapping(body, &provider, &AppType::Codex);
        assert_eq!(result["model"], "gpt-5.3-codex-high");
        assert_eq!(original, Some("anything-client-sent".to_string()));
        assert_eq!(mapped, Some("gpt-5.3-codex-high".to_string()));
    }

    #[test]
    fn test_codex_force_model_mapping_when_model_missing() {
        let provider = create_codex_provider_with_model("gpt-5.3-codex-high");
        let body = json!({"messages": [{"role":"user","content":"hi"}]});
        let (result, original, mapped) = apply_model_mapping(body, &provider, &AppType::Codex);
        assert_eq!(result["model"], "gpt-5.3-codex-high");
        assert!(original.is_none());
        assert_eq!(mapped, Some("gpt-5.3-codex-high".to_string()));
    }

    #[test]
    fn test_codex_no_force_model_keeps_original() {
        let provider = create_codex_provider_without_model();
        let body = json!({"model": "client-model"});
        let (result, original, mapped) = apply_model_mapping(body, &provider, &AppType::Codex);
        assert_eq!(result["model"], "client-model");
        assert_eq!(original, Some("client-model".to_string()));
        assert!(mapped.is_none());
    }

    #[test]
    fn test_haiku_mapping() {
        let provider = create_provider_with_mapping();
        let body = json!({"model": "claude-haiku-4-5"});
        let (result, _, mapped) = apply_model_mapping(body, &provider, &AppType::Claude);
        assert_eq!(result["model"], "haiku-mapped");
        assert_eq!(mapped, Some("haiku-mapped".to_string()));
    }

    #[test]
    fn test_opus_mapping() {
        let provider = create_provider_with_mapping();
        let body = json!({"model": "claude-opus-4-5"});
        let (result, _, mapped) = apply_model_mapping(body, &provider, &AppType::Claude);
        assert_eq!(result["model"], "opus-mapped");
        assert_eq!(mapped, Some("opus-mapped".to_string()));
    }

    #[test]
    fn test_thinking_mode() {
        let provider = create_provider_with_mapping();
        let body = json!({
            "model": "claude-sonnet-4-5",
            "thinking": {"type": "enabled"}
        });
        let (result, _, mapped) = apply_model_mapping(body, &provider, &AppType::Claude);
        assert_eq!(result["model"], "reasoning-model");
        assert_eq!(mapped, Some("reasoning-model".to_string()));
    }

    #[test]
    fn test_reasoning_only_mapping_in_thinking_mode() {
        let provider = create_provider_with_reasoning_only();
        let body = json!({
            "model": "claude-sonnet-4-5",
            "thinking": {"type": "enabled"}
        });
        let (result, _, mapped) = apply_model_mapping(body, &provider, &AppType::Claude);
        assert_eq!(result["model"], "reasoning-only-model");
        assert_eq!(mapped, Some("reasoning-only-model".to_string()));
    }

    #[test]
    fn test_reasoning_only_mapping_does_not_affect_non_thinking() {
        let provider = create_provider_with_reasoning_only();
        let body = json!({
            "model": "claude-sonnet-4-5",
            "thinking": {"type": "disabled"}
        });
        let (result, original, mapped) = apply_model_mapping(body, &provider, &AppType::Claude);
        assert_eq!(result["model"], "claude-sonnet-4-5");
        assert_eq!(original, Some("claude-sonnet-4-5".to_string()));
        assert!(mapped.is_none());
    }

    #[test]
    fn test_thinking_disabled() {
        let provider = create_provider_with_mapping();
        let body = json!({
            "model": "claude-sonnet-4-5",
            "thinking": {"type": "disabled"}
        });
        let (result, _, mapped) = apply_model_mapping(body, &provider, &AppType::Claude);
        assert_eq!(result["model"], "sonnet-mapped");
        assert_eq!(mapped, Some("sonnet-mapped".to_string()));
    }

    #[test]
    fn test_unknown_model_uses_default() {
        let provider = create_provider_with_mapping();
        let body = json!({"model": "some-unknown-model"});
        let (result, _, mapped) = apply_model_mapping(body, &provider, &AppType::Claude);
        assert_eq!(result["model"], "default-model");
        assert_eq!(mapped, Some("default-model".to_string()));
    }

    #[test]
    fn test_no_mapping_configured() {
        let provider = create_provider_without_mapping();
        let body = json!({"model": "claude-sonnet-4-5"});
        let (result, original, mapped) = apply_model_mapping(body, &provider, &AppType::Claude);
        assert_eq!(result["model"], "claude-sonnet-4-5");
        assert_eq!(original, Some("claude-sonnet-4-5".to_string()));
        assert!(mapped.is_none());
    }

    #[test]
    fn test_thinking_adaptive() {
        let provider = create_provider_with_mapping();
        let body = json!({
            "model": "claude-sonnet-4-5",
            "thinking": {"type": "adaptive"}
        });
        let (result, _, mapped) = apply_model_mapping(body, &provider, &AppType::Claude);
        assert_eq!(result["model"], "reasoning-model");
        assert_eq!(mapped, Some("reasoning-model".to_string()));
    }

    #[test]
    fn test_thinking_unknown_type() {
        let provider = create_provider_with_mapping();
        let body = json!({
            "model": "claude-sonnet-4-5",
            "thinking": {"type": "some_future_type"}
        });
        let (result, _, mapped) = apply_model_mapping(body, &provider, &AppType::Claude);
        assert_eq!(result["model"], "sonnet-mapped");
        assert_eq!(mapped, Some("sonnet-mapped".to_string()));
    }

    #[test]
    fn test_case_insensitive() {
        let provider = create_provider_with_mapping();
        let body = json!({"model": "Claude-SONNET-4-5"});
        let (result, _, mapped) = apply_model_mapping(body, &provider, &AppType::Claude);
        assert_eq!(result["model"], "sonnet-mapped");
        assert_eq!(mapped, Some("sonnet-mapped".to_string()));
    }
}

