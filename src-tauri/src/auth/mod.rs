//! Accounts: offline (UUID v3) + Microsoft OAuth (in-app window, desktop
//! redirect), stored locally in `accounts.json`. No secrets in code — the
//! Microsoft client id is the public one historically used by launchers.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};
use anyhow::Result;
use reqwest::Client;

pub mod microsoft;
pub mod offline;

pub use microsoft::MicrosoftAuth;
pub use offline::OfflineAuth;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: String,
    pub username: String,
    pub uuid: String,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub token_expires_at: Option<DateTime<Utc>>,
    pub account_type: AccountType,
    pub created_at: DateTime<Utc>,
    pub last_used: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AccountType {
    Microsoft,
    Offline,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MicrosoftTokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u64,
    pub refresh_token: String,
    pub scope: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XboxLiveResponse {
    #[serde(rename = "IssueInstant")]
    pub issue_instant: String,
    #[serde(rename = "NotAfter")]
    pub not_after: String,
    #[serde(rename = "Token")]
    pub token: String,
    #[serde(rename = "DisplayClaims")]
    pub display_claims: DisplayClaims,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayClaims {
    pub xui: Vec<XuiClaim>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XuiClaim {
    pub uhs: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XstsResponse {
    #[serde(rename = "IssueInstant")]
    pub issue_instant: String,
    #[serde(rename = "NotAfter")]
    pub not_after: String,
    #[serde(rename = "Token")]
    pub token: String,
    #[serde(rename = "DisplayClaims")]
    pub display_claims: DisplayClaims,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinecraftAuthResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinecraftProfile {
    pub id: String,
    pub name: String,
}

pub struct AuthManager {
    accounts: Arc<RwLock<HashMap<String, Account>>>,
    current_account: Arc<RwLock<Option<String>>>,
    #[allow(dead_code)]
    client: Client,
    microsoft_auth: MicrosoftAuth,
    offline_auth: OfflineAuth,
    store_path: std::path::PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct AuthStoreFile {
    #[serde(default)]
    accounts: Vec<Account>,
    #[serde(default)]
    current_account: Option<String>,
}

impl AuthManager {
    pub async fn new() -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .unwrap();

        let store_path = crate::common::base_dir().join("accounts.json");
        if let Some(parent) = store_path.parent() {
            let _ = tokio::fs::create_dir_all(parent).await;
        }

        Self {
            accounts: Arc::new(RwLock::new(HashMap::new())),
            current_account: Arc::new(RwLock::new(None)),
            client: client.clone(),
            microsoft_auth: MicrosoftAuth::new(client),
            offline_auth: OfflineAuth::new(),
            store_path,
        }
    }

    pub async fn load_accounts(&self) -> Result<()> {
        if !self.store_path.exists() {
            return Ok(());
        }
        let content = tokio::fs::read_to_string(&self.store_path).await?;
        if content.trim().is_empty() {
            return Ok(());
        }
        let file: AuthStoreFile = serde_json::from_str(&content).unwrap_or_default();
        let mut accounts = self.accounts.write().await;
        accounts.clear();
        for acc in file.accounts {
            accounts.insert(acc.id.clone(), acc);
        }
        drop(accounts);
        let mut current = self.current_account.write().await;
        *current = file.current_account;
        // fallback: pick first account if current missing
        if current.is_none() {
            let accounts = self.accounts.read().await;
            *current = accounts.keys().next().cloned();
        }
        Ok(())
    }

    pub async fn save_accounts(&self) -> Result<()> {
        let accounts = self.accounts.read().await;
        let current = self.current_account.read().await;
        let file = AuthStoreFile {
            accounts: accounts.values().cloned().collect(),
            current_account: current.clone(),
        };
        let content = serde_json::to_string_pretty(&file)?;
        tokio::fs::write(&self.store_path, content).await?;
        Ok(())
    }

    pub async fn touch_last_used(&self, account_id: &str) -> Result<()> {
        let mut accounts = self.accounts.write().await;
        if let Some(acc) = accounts.get_mut(account_id) {
            acc.last_used = Utc::now();
        }
        drop(accounts);
        self.save_accounts().await?;
        Ok(())
    }
}

#[tauri::command]
pub async fn login_microsoft(
    state: tauri::State<'_, crate::AppState>,
    app: tauri::AppHandle,
) -> Result<Account, String> {
    let account = state
        .auth
        .microsoft_auth
        .authenticate(app)
        .await
        .map_err(|e| e.to_string())?;

    {
        let mut accounts = state.auth.accounts.write().await;
        accounts.insert(account.id.clone(), account.clone());
    }
    {
        let mut current = state.auth.current_account.write().await;
        *current = Some(account.id.clone());
    }
    state.auth.save_accounts().await.map_err(|e| e.to_string())?;
    Ok(account)
}

#[tauri::command]
pub async fn login_offline(
    state: tauri::State<'_, crate::AppState>,
    username: String,
) -> Result<Account, String> {
    let account = state
        .auth
        .offline_auth
        .authenticate(username)
        .await
        .map_err(|e| e.to_string())?;

    {
        let mut accounts = state.auth.accounts.write().await;
        // reuse existing offline account with same username if present
        if let Some(existing) = accounts
            .values()
            .find(|a| a.account_type == AccountType::Offline && a.username == account.username)
            .cloned()
        {
            let mut current = state.auth.current_account.write().await;
            *current = Some(existing.id.clone());
            state.auth.save_accounts().await.map_err(|e| e.to_string())?;
            return Ok(existing);
        }
        accounts.insert(account.id.clone(), account.clone());
    }
    {
        let mut current = state.auth.current_account.write().await;
        *current = Some(account.id.clone());
    }
    state.auth.save_accounts().await.map_err(|e| e.to_string())?;
    Ok(account)
}

#[tauri::command]
pub async fn logout(
    state: tauri::State<'_, crate::AppState>,
    account_id: String,
) -> Result<(), String> {
    {
        let mut accounts = state.auth.accounts.write().await;
        accounts.remove(&account_id);
    }
    {
        let mut current = state.auth.current_account.write().await;
        if current.as_ref() == Some(&account_id) {
            let accounts = state.auth.accounts.read().await;
            *current = accounts.keys().next().cloned();
        }
    }
    state.auth.save_accounts().await.map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn get_accounts(
    state: tauri::State<'_, crate::AppState>,
) -> Result<Vec<Account>, String> {
    let accounts = state.auth.accounts.read().await;
    let mut list: Vec<Account> = accounts.values().cloned().collect();
    list.sort_by(|a, b| b.last_used.cmp(&a.last_used));
    Ok(list)
}

#[tauri::command]
pub async fn get_current_account(
    state: tauri::State<'_, crate::AppState>,
) -> Result<Option<Account>, String> {
    let current = state.auth.current_account.read().await.clone();
    if let Some(id) = current {
        let accounts = state.auth.accounts.read().await;
        Ok(accounts.get(&id).cloned())
    } else {
        Ok(None)
    }
}

#[tauri::command]
pub async fn set_current_account(
    state: tauri::State<'_, crate::AppState>,
    account_id: String,
) -> Result<Account, String> {
    let acc = {
        let accounts = state.auth.accounts.read().await;
        accounts
            .get(&account_id)
            .cloned()
            .ok_or_else(|| "Account not found".to_string())?
    };
    {
        let mut current = state.auth.current_account.write().await;
        *current = Some(account_id);
    }
    state.auth.save_accounts().await.map_err(|e| e.to_string())?;
    Ok(acc)
}

#[tauri::command]
pub async fn refresh_token(
    state: tauri::State<'_, crate::AppState>,
    account_id: String,
) -> Result<Account, String> {
    let stored = {
        let accounts = state.auth.accounts.read().await;
        accounts
            .get(&account_id)
            .cloned()
            .ok_or_else(|| "Account not found".to_string())?
    };
    if stored.account_type != AccountType::Microsoft {
        return Err("Cannot refresh offline account".to_string());
    }
    let refreshed = state
        .auth
        .microsoft_auth
        .refresh_token(&stored)
        .await
        .map_err(|e| e.to_string())?;
    {
        let mut accounts = state.auth.accounts.write().await;
        accounts.insert(account_id, refreshed.clone());
    }
    state.auth.save_accounts().await.map_err(|e| e.to_string())?;
    Ok(refreshed)
}

#[tauri::command]
pub async fn validate_account(
    state: tauri::State<'_, crate::AppState>,
    account_id: String,
) -> Result<bool, String> {
    let account = {
        let accounts = state.auth.accounts.read().await;
        accounts
            .get(&account_id)
            .cloned()
            .ok_or_else(|| "Account not found".to_string())?
    };
    if account.account_type == AccountType::Microsoft {
        Ok(state
            .auth
            .microsoft_auth
            .validate_token(&account)
            .await
            .unwrap_or(false))
    } else {
        Ok(true)
    }
}
