use std::{
    fs, io,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use thiserror::Error;

pub type Result<T> = std::result::Result<T, SwitchyardError>;

#[derive(Debug, Error)]
pub enum SwitchyardError {
    #[error("未找到主目录，无法定位 .codex")]
    HomeDirMissing,
    #[error("无法读取文件 {path}: {source}")]
    ReadFile { path: PathBuf, source: io::Error },
    #[error("无法写入文件 {path}: {source}")]
    WriteFile { path: PathBuf, source: io::Error },
    #[error("无法解析 JSON {path}: {source}")]
    ParseJson {
        path: PathBuf,
        source: serde_json::Error,
    },
    #[error("无法序列化注册表: {0}")]
    EncodeJson(serde_json::Error),
    #[error("未找到注册表文件: {path}")]
    MissingRegistry { path: PathBuf },
    #[error("未找到账号: {account_key}")]
    AccountNotFound { account_key: String },
    #[error("未找到账号快照文件: {path}")]
    MissingSnapshot { path: PathBuf },
    #[error("auth.json 格式不完整，缺少必要账号字段")]
    IncompleteAuth,
    #[error("auth.json 中的 account_id 与 JWT claim 不一致")]
    AccountIdMismatch,
    #[error("auth.json 不是合法的 JWT/JSON 格式")]
    InvalidAuthFormat,
    #[error("无法解码 JWT 载荷")]
    InvalidJwt,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct RegistryData {
    #[serde(default = "default_schema_version")]
    pub schema_version: u32,
    pub active_account_key: Option<String>,
    pub active_account_activated_at_ms: Option<i64>,
    pub auto_switch: AutoSwitchConfig,
    pub api: ApiConfig,
    pub accounts: Vec<AccountRecord>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl Default for RegistryData {
    fn default() -> Self {
        Self {
            schema_version: default_schema_version(),
            active_account_key: None,
            active_account_activated_at_ms: None,
            auto_switch: AutoSwitchConfig::default(),
            api: ApiConfig::default(),
            accounts: Vec::new(),
            extra: Map::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AutoSwitchConfig {
    pub enabled: bool,
    #[serde(default = "default_threshold_5h", rename = "threshold_5h_percent")]
    pub threshold_5h_percent: i32,
    #[serde(
        default = "default_threshold_weekly",
        rename = "threshold_weekly_percent"
    )]
    pub threshold_weekly_percent: i32,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl Default for AutoSwitchConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            threshold_5h_percent: default_threshold_5h(),
            threshold_weekly_percent: default_threshold_weekly(),
            extra: Map::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ApiConfig {
    #[serde(default = "default_true")]
    pub usage: bool,
    #[serde(default = "default_true")]
    pub account: bool,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            usage: true,
            account: true,
            extra: Map::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct AccountRecord {
    pub account_key: String,
    pub chatgpt_account_id: String,
    pub chatgpt_user_id: String,
    pub email: String,
    pub alias: String,
    pub account_name: Option<String>,
    pub plan: Option<String>,
    pub auth_mode: Option<String>,
    pub created_at: i64,
    pub last_used_at: Option<i64>,
    pub last_usage: Option<RateLimitSnapshot>,
    pub last_usage_at: Option<i64>,
    pub last_local_rollout: Option<RolloutSignature>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct RateLimitSnapshot {
    pub primary: Option<RateLimitWindow>,
    pub secondary: Option<RateLimitWindow>,
    pub credits: Option<CreditsSnapshot>,
    pub plan_type: Option<String>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct RateLimitWindow {
    pub used_percent: f64,
    pub window_minutes: Option<i64>,
    pub resets_at: Option<i64>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct CreditsSnapshot {
    pub has_credits: bool,
    pub unlimited: bool,
    pub balance: Option<String>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct RolloutSignature {
    pub path: String,
    pub event_timestamp_ms: i64,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PanelState {
    pub codex_home: String,
    pub registry_path: String,
    pub has_registry: bool,
    pub active_account_key: Option<String>,
    pub auto_switch_enabled: bool,
    pub api_usage_enabled: bool,
    pub accounts: Vec<AccountSummary>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountSummary {
    pub account_key: String,
    pub email: String,
    pub alias: String,
    pub account_name: Option<String>,
    pub display_name: String,
    pub plan: Option<String>,
    pub is_active: bool,
    pub usage_5h_remaining: Option<i32>,
    pub usage_weekly_remaining: Option<i32>,
    pub has_auth_snapshot: bool,
    pub last_used_at: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct AuthInfo {
    pub email: Option<String>,
    pub chatgpt_account_id: Option<String>,
    pub chatgpt_user_id: Option<String>,
    pub record_key: Option<String>,
    pub access_token: Option<String>,
    pub last_refresh: Option<String>,
    pub plan: Option<String>,
    pub auth_mode: String,
}

pub fn resolve_codex_home() -> Result<PathBuf> {
    if let Ok(path) = std::env::var("SWITCHYARD_CODEX_HOME") {
        if !path.trim().is_empty() {
            return Ok(PathBuf::from(path));
        }
    }

    dirs::home_dir()
        .map(|path| path.join(".codex"))
        .ok_or(SwitchyardError::HomeDirMissing)
}

pub fn accounts_dir(codex_home: &Path) -> PathBuf {
    codex_home.join("accounts")
}

pub fn registry_path(codex_home: &Path) -> PathBuf {
    accounts_dir(codex_home).join("registry.json")
}

pub fn active_auth_path(codex_home: &Path) -> PathBuf {
    codex_home.join("auth.json")
}

pub fn account_file_key(account_key: &str) -> String {
    if account_key.is_empty() || account_key == "." || account_key == ".." {
        return URL_SAFE_NO_PAD.encode(account_key.as_bytes());
    }

    if account_key
        .chars()
        .all(|value| value.is_ascii_alphanumeric() || matches!(value, '-' | '_' | '.'))
    {
        return account_key.to_string();
    }

    URL_SAFE_NO_PAD.encode(account_key.as_bytes())
}

pub fn account_auth_path(codex_home: &Path, account_key: &str) -> PathBuf {
    accounts_dir(codex_home).join(format!("{}.auth.json", account_file_key(account_key)))
}

pub fn load_registry(codex_home: &Path) -> Result<RegistryData> {
    let path = registry_path(codex_home);
    let bytes = fs::read(&path).map_err(|source| SwitchyardError::ReadFile {
        path: path.clone(),
        source,
    })?;
    serde_json::from_slice(&bytes).map_err(|source| SwitchyardError::ParseJson { path, source })
}

pub fn save_registry(codex_home: &Path, registry: &RegistryData) -> Result<()> {
    let path = registry_path(codex_home);
    let payload = serde_json::to_vec_pretty(registry).map_err(SwitchyardError::EncodeJson)?;
    atomic_write(&path, &payload)
}

pub fn load_panel_state(codex_home: &Path) -> Result<PanelState> {
    let registry_path = registry_path(codex_home);
    if !registry_path.exists() {
        return Ok(PanelState {
            codex_home: codex_home.display().to_string(),
            registry_path: registry_path.display().to_string(),
            has_registry: false,
            active_account_key: None,
            auto_switch_enabled: false,
            api_usage_enabled: true,
            accounts: Vec::new(),
        });
    }

    let registry = load_registry(codex_home)?;
    Ok(panel_state_from_registry(codex_home, &registry))
}

pub fn switch_account(codex_home: &Path, account_key: &str) -> Result<PanelState> {
    let mut registry = load_registry(codex_home)?;
    let account_exists = registry
        .accounts
        .iter()
        .any(|account| account.account_key == account_key);
    if !account_exists {
        return Err(SwitchyardError::AccountNotFound {
            account_key: account_key.to_string(),
        });
    }

    let _ = sync_active_account_snapshot(codex_home, &registry);

    let source = account_auth_path(codex_home, account_key);
    if !source.exists() {
        return Err(SwitchyardError::MissingSnapshot { path: source });
    }

    atomic_copy(&source, &active_auth_path(codex_home))?;

    let now_ms = now_unix_ms();
    let now_seconds = now_ms / 1000;
    registry.active_account_key = Some(account_key.to_string());
    registry.active_account_activated_at_ms = Some(now_ms);

    if let Some(account) = registry
        .accounts
        .iter_mut()
        .find(|account| account.account_key == account_key)
    {
        account.last_used_at = Some(now_seconds);
    }

    save_registry(codex_home, &registry)?;
    Ok(panel_state_from_registry(codex_home, &registry))
}

pub fn set_auto_switch_enabled(codex_home: &Path, enabled: bool) -> Result<PanelState> {
    let mut registry = load_registry(codex_home)?;
    registry.auto_switch.enabled = enabled;
    save_registry(codex_home, &registry)?;
    Ok(panel_state_from_registry(codex_home, &registry))
}

pub fn set_usage_api_enabled(codex_home: &Path, enabled: bool) -> Result<PanelState> {
    let mut registry = load_registry(codex_home)?;
    registry.api.usage = enabled;
    registry.api.account = enabled;
    save_registry(codex_home, &registry)?;
    Ok(panel_state_from_registry(codex_home, &registry))
}

pub fn parse_auth_file(path: &Path) -> Result<AuthInfo> {
    let bytes = fs::read(path).map_err(|source| SwitchyardError::ReadFile {
        path: path.to_path_buf(),
        source,
    })?;
    parse_auth_bytes(&bytes)
}

pub fn parse_auth_bytes(bytes: &[u8]) -> Result<AuthInfo> {
    let root: Value =
        serde_json::from_slice(bytes).map_err(|source| SwitchyardError::ParseJson {
            path: PathBuf::from("auth.json"),
            source,
        })?;

    if let Some(api_key) = root
        .get("OPENAI_API_KEY")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
    {
        return Ok(AuthInfo {
            email: None,
            chatgpt_account_id: None,
            chatgpt_user_id: None,
            record_key: None,
            access_token: Some(api_key.to_string()),
            last_refresh: string_field(&root, "last_refresh"),
            plan: None,
            auth_mode: "apikey".to_string(),
        });
    }

    let tokens = root
        .get("tokens")
        .and_then(Value::as_object)
        .ok_or(SwitchyardError::IncompleteAuth)?;
    let id_token = tokens
        .get("id_token")
        .and_then(Value::as_str)
        .ok_or(SwitchyardError::IncompleteAuth)?;
    let payload = decode_jwt_payload(id_token)?;
    let claims: Value =
        serde_json::from_slice(&payload).map_err(|_| SwitchyardError::InvalidAuthFormat)?;
    let auth_claims = claims
        .get("https://api.openai.com/auth")
        .and_then(Value::as_object)
        .ok_or(SwitchyardError::IncompleteAuth)?;

    let token_account_id = tokens
        .get("account_id")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned);
    let claim_account_id = auth_claims
        .get("chatgpt_account_id")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned);

    if let (Some(token_account_id), Some(claim_account_id)) =
        (token_account_id.as_ref(), claim_account_id.as_ref())
    {
        if token_account_id != claim_account_id {
            return Err(SwitchyardError::AccountIdMismatch);
        }
    }

    let chatgpt_account_id = token_account_id.or(claim_account_id);
    let chatgpt_user_id = auth_claims
        .get("chatgpt_user_id")
        .and_then(Value::as_str)
        .or_else(|| auth_claims.get("user_id").and_then(Value::as_str))
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned);

    let record_key = match (chatgpt_user_id.as_ref(), chatgpt_account_id.as_ref()) {
        (Some(user_id), Some(account_id)) => Some(format!("{user_id}::{account_id}")),
        _ => None,
    };

    Ok(AuthInfo {
        email: claims
            .get("email")
            .and_then(Value::as_str)
            .map(|value| value.to_ascii_lowercase()),
        chatgpt_account_id,
        chatgpt_user_id,
        record_key,
        access_token: tokens
            .get("access_token")
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned),
        last_refresh: string_field(&root, "last_refresh"),
        plan: auth_claims
            .get("chatgpt_plan_type")
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned),
        auth_mode: "chatgpt".to_string(),
    })
}

fn panel_state_from_registry(codex_home: &Path, registry: &RegistryData) -> PanelState {
    let mut accounts: Vec<AccountSummary> = registry
        .accounts
        .iter()
        .map(|account| account_summary(codex_home, registry.active_account_key.as_deref(), account))
        .collect();

    accounts.sort_by(|left, right| {
        right
            .is_active
            .cmp(&left.is_active)
            .then_with(|| left.email.cmp(&right.email))
    });

    PanelState {
        codex_home: codex_home.display().to_string(),
        registry_path: registry_path(codex_home).display().to_string(),
        has_registry: true,
        active_account_key: registry.active_account_key.clone(),
        auto_switch_enabled: registry.auto_switch.enabled,
        api_usage_enabled: registry.api.usage,
        accounts,
    }
}

fn account_summary(
    codex_home: &Path,
    active_account_key: Option<&str>,
    account: &AccountRecord,
) -> AccountSummary {
    let usage_5h = account
        .last_usage
        .as_ref()
        .and_then(|snapshot| resolve_window(snapshot, 300, true))
        .map(remaining_percent);
    let usage_weekly = account
        .last_usage
        .as_ref()
        .and_then(|snapshot| resolve_window(snapshot, 10080, false))
        .map(remaining_percent);

    let plan = account.plan.clone().or_else(|| {
        account
            .last_usage
            .as_ref()
            .and_then(|snapshot| snapshot.plan_type.clone())
    });

    AccountSummary {
        account_key: account.account_key.clone(),
        email: account.email.clone(),
        alias: account.alias.clone(),
        account_name: account.account_name.clone(),
        display_name: display_name(account),
        plan,
        is_active: active_account_key == Some(account.account_key.as_str()),
        usage_5h_remaining: usage_5h,
        usage_weekly_remaining: usage_weekly,
        has_auth_snapshot: account_auth_path(codex_home, &account.account_key).exists(),
        last_used_at: account.last_used_at,
    }
}

fn display_name(account: &AccountRecord) -> String {
    if !account.alias.trim().is_empty() {
        return format!("{} · {}", account.email, account.alias.trim());
    }

    if let Some(account_name) = account
        .account_name
        .as_ref()
        .filter(|value| !value.trim().is_empty())
    {
        return format!("{} · {}", account.email, account_name.trim());
    }

    account.email.clone()
}

fn sync_active_account_snapshot(codex_home: &Path, registry: &RegistryData) -> Result<()> {
    let Some(active_account_key) = registry.active_account_key.as_deref() else {
        return Ok(());
    };

    let auth_path = active_auth_path(codex_home);
    if !auth_path.exists() {
        return Ok(());
    }

    let auth_info = parse_auth_file(&auth_path)?;
    let Some(record_key) = auth_info.record_key.as_deref() else {
        return Ok(());
    };

    if record_key != active_account_key {
        return Ok(());
    }

    let target = account_auth_path(codex_home, active_account_key);
    atomic_copy(&auth_path, &target)
}

fn resolve_window(
    snapshot: &RateLimitSnapshot,
    window_minutes: i64,
    fallback_primary: bool,
) -> Option<&RateLimitWindow> {
    if snapshot
        .primary
        .as_ref()
        .is_some_and(|window| window.window_minutes == Some(window_minutes))
    {
        return snapshot.primary.as_ref();
    }

    if snapshot
        .secondary
        .as_ref()
        .is_some_and(|window| window.window_minutes == Some(window_minutes))
    {
        return snapshot.secondary.as_ref();
    }

    if fallback_primary {
        snapshot.primary.as_ref()
    } else {
        snapshot.secondary.as_ref()
    }
}

fn remaining_percent(window: &RateLimitWindow) -> i32 {
    let now_seconds = now_unix_ms() / 1000;
    if window
        .resets_at
        .is_some_and(|timestamp| timestamp <= now_seconds)
    {
        return 100;
    }

    let remaining = (100.0 - window.used_percent).round() as i32;
    remaining.clamp(0, 100)
}

fn string_field(root: &Value, field: &str) -> Option<String> {
    root.get(field)
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn decode_jwt_payload(jwt: &str) -> Result<Vec<u8>> {
    let mut parts = jwt.split('.');
    let _header = parts.next();
    let payload = parts.next().ok_or(SwitchyardError::InvalidJwt)?;
    let _signature = parts.next().ok_or(SwitchyardError::InvalidJwt)?;
    URL_SAFE_NO_PAD
        .decode(payload)
        .map_err(|_| SwitchyardError::InvalidJwt)
}

fn atomic_copy(source: &Path, target: &Path) -> Result<()> {
    let payload = fs::read(source).map_err(|source_error| SwitchyardError::ReadFile {
        path: source.to_path_buf(),
        source: source_error,
    })?;
    atomic_write(target, &payload)
}

fn atomic_write(path: &Path, payload: &[u8]) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| SwitchyardError::WriteFile {
            path: parent.to_path_buf(),
            source,
        })?;
    }

    let temp_path = temp_path_for(path);
    fs::write(&temp_path, payload).map_err(|source| SwitchyardError::WriteFile {
        path: temp_path.clone(),
        source,
    })?;

    if path.exists() {
        fs::remove_file(path).map_err(|source| SwitchyardError::WriteFile {
            path: path.to_path_buf(),
            source,
        })?;
    }

    fs::rename(&temp_path, path).map_err(|source| SwitchyardError::WriteFile {
        path: path.to_path_buf(),
        source,
    })
}

fn temp_path_for(path: &Path) -> PathBuf {
    let file_name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("switchyard.tmp");
    path.with_file_name(format!(".{file_name}.{}.tmp", now_unix_ms()))
}

fn now_unix_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}

fn default_schema_version() -> u32 {
    3
}

fn default_threshold_5h() -> i32 {
    10
}

fn default_threshold_weekly() -> i32 {
    5
}

fn default_true() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static TEMP_COUNTER: AtomicUsize = AtomicUsize::new(0);

    fn fixture_auth(email: &str, user_id: &str, account_id: &str, plan: &str) -> String {
        let payload = serde_json::json!({
            "email": email,
            "https://api.openai.com/auth": {
                "chatgpt_account_id": account_id,
                "chatgpt_user_id": user_id,
                "chatgpt_plan_type": plan
            }
        });
        let payload = URL_SAFE_NO_PAD.encode(payload.to_string().as_bytes());
        serde_json::json!({
            "auth_mode": "chatgpt",
            "tokens": {
                "id_token": format!("header.{payload}.signature"),
                "access_token": "token-value",
                "refresh_token": "refresh-value",
                "account_id": account_id
            },
            "last_refresh": "2026-04-09T12:00:00Z"
        })
        .to_string()
    }

    fn temp_codex_home() -> PathBuf {
        let counter = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "switchyard-core-{}-{}-{}",
            std::process::id(),
            now_unix_ms(),
            counter
        ));
        fs::create_dir_all(accounts_dir(&path)).unwrap();
        path
    }

    fn write_registry_fixture(codex_home: &Path) -> RegistryData {
        let registry = RegistryData {
            active_account_key: Some("user-a::acct-a".to_string()),
            active_account_activated_at_ms: Some(1_775_723_221_000),
            accounts: vec![
                AccountRecord {
                    account_key: "user-a::acct-a".to_string(),
                    chatgpt_account_id: "acct-a".to_string(),
                    chatgpt_user_id: "user-a".to_string(),
                    email: "alpha@example.com".to_string(),
                    alias: "personal".to_string(),
                    plan: Some("plus".to_string()),
                    created_at: 1,
                    last_usage: Some(RateLimitSnapshot {
                        primary: Some(RateLimitWindow {
                            used_percent: 35.0,
                            window_minutes: Some(300),
                            resets_at: Some((now_unix_ms() / 1000) + 3600),
                            extra: Map::new(),
                        }),
                        secondary: Some(RateLimitWindow {
                            used_percent: 55.0,
                            window_minutes: Some(10080),
                            resets_at: Some((now_unix_ms() / 1000) + 7200),
                            extra: Map::new(),
                        }),
                        plan_type: Some("plus".to_string()),
                        ..RateLimitSnapshot::default()
                    }),
                    ..AccountRecord::default()
                },
                AccountRecord {
                    account_key: "user-b::acct-b".to_string(),
                    chatgpt_account_id: "acct-b".to_string(),
                    chatgpt_user_id: "user-b".to_string(),
                    email: "beta@example.com".to_string(),
                    plan: Some("team".to_string()),
                    created_at: 2,
                    ..AccountRecord::default()
                },
            ],
            ..RegistryData::default()
        };
        save_registry(codex_home, &registry).unwrap();
        registry
    }

    #[test]
    fn account_file_key_encodes_special_keys() {
        let encoded = account_file_key("user-a::acct-a");
        assert_ne!(encoded, "user-a::acct-a");
        assert_eq!(account_file_key("simple_key"), "simple_key");
    }

    #[test]
    fn load_panel_state_sorts_active_account_first() {
        let codex_home = temp_codex_home();
        write_registry_fixture(&codex_home);
        fs::write(
            account_auth_path(&codex_home, "user-a::acct-a"),
            fixture_auth("alpha@example.com", "user-a", "acct-a", "plus"),
        )
        .unwrap();

        let state = load_panel_state(&codex_home).unwrap();
        assert!(state.has_registry);
        assert_eq!(state.accounts.len(), 2);
        assert!(state.accounts[0].is_active);
        assert_eq!(state.accounts[0].email, "alpha@example.com");
        assert_eq!(state.accounts[0].usage_5h_remaining, Some(65));
    }

    #[test]
    fn switch_account_updates_active_auth_and_registry() {
        let codex_home = temp_codex_home();
        write_registry_fixture(&codex_home);

        fs::write(
            active_auth_path(&codex_home),
            fixture_auth("alpha@example.com", "user-a", "acct-a", "plus"),
        )
        .unwrap();
        fs::write(
            account_auth_path(&codex_home, "user-a::acct-a"),
            fixture_auth("alpha@example.com", "user-a", "acct-a", "plus"),
        )
        .unwrap();
        fs::write(
            account_auth_path(&codex_home, "user-b::acct-b"),
            fixture_auth("beta@example.com", "user-b", "acct-b", "team"),
        )
        .unwrap();

        let state = switch_account(&codex_home, "user-b::acct-b").unwrap();
        let registry = load_registry(&codex_home).unwrap();
        let auth = parse_auth_file(&active_auth_path(&codex_home)).unwrap();

        assert_eq!(state.active_account_key.as_deref(), Some("user-b::acct-b"));
        assert_eq!(
            registry.active_account_key.as_deref(),
            Some("user-b::acct-b")
        );
        assert_eq!(auth.record_key.as_deref(), Some("user-b::acct-b"));
    }
}
