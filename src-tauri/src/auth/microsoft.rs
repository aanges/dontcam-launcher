use super::*;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use anyhow::Result;
use reqwest::Client;
use chrono::Utc;
use uuid::Uuid;
use tauri::{Emitter, Manager};

// Public client ID historically used by Minecraft launchers (no secret needed).
// NOTE: this app only allows the official desktop redirect below —
// arbitrary localhost callbacks are rejected by Microsoft (AADSTS invalid_request).
const CLIENT_ID: &str = "00000000402b5328";
const AUTH_URL: &str = "https://login.live.com/oauth20_authorize.srf";
const TOKEN_URL: &str = "https://login.live.com/oauth20_token.srf";
const DESKTOP_REDIRECT: &str = "https://login.live.com/oauth20_desktop.srf";
const SCOPE: &str = "XboxLive.signin offline_access";

// Public client ID historically used by Minecraft launchers (no secret needed).
const XBL_AUTH_URL: &str = "https://user.auth.xboxlive.com/user/authenticate";
const XSTS_AUTH_URL: &str = "https://xsts.auth.xboxlive.com/xsts/authorize";
const MINECRAFT_AUTH_URL: &str =
    "https://api.minecraftservices.com/authentication/login_with_xbox";
const MINECRAFT_PROFILE_URL: &str = "https://api.minecraftservices.com/minecraft/profile";
const MINECRAFT_ENTITLEMENTS_URL: &str =
    "https://api.minecraftservices.com/entitlements/mcstore";

#[derive(Clone)]
pub struct MicrosoftAuth {
    client: Client,
}

impl MicrosoftAuth {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    /// Full login: embedded sign-in window, capture the code from the
    /// official desktop redirect, then exchange it for tokens.
    pub async fn authenticate(&self, app: tauri::AppHandle) -> Result<Account> {
        use tauri::{WebviewUrl, WebviewWindowBuilder};

        let state = Uuid::new_v4().to_string();
        let auth_url = format!(
            "{}?client_id={}&response_type=code&redirect_uri={}&scope={}&state={}&prompt=select_account",
            AUTH_URL,
            CLIENT_ID,
            percent_encode(DESKTOP_REDIRECT),
            percent_encode(SCOPE),
            state,
        );

        // Fallback visibility: frontend can show this URL if the window fails.
        let _ = app.emit(
            "ms-login-url",
            serde_json::json!({ "url": auth_url }),
        );

        let window = WebviewWindowBuilder::new(
            &app,
            "microsoft-auth",
            WebviewUrl::External(auth_url.parse().map_err(|e| anyhow::anyhow!("Bad auth URL: {}", e))?),
        )
        .title("Sign in with Microsoft")
        .inner_size(520.0, 700.0)
        .resizable(true)
        .center()
        .build()
        .map_err(|e| anyhow::anyhow!("Failed to open sign-in window: {}", e))?;

        let deadline = Instant::now() + Duration::from_secs(300);
        loop {
            if Instant::now() > deadline {
                let _ = window.close();
                return Err(anyhow::anyhow!("Login timed out after 5 minutes"));
            }
            // Closed by the user = cancelled.
            if app.get_webview_window("microsoft-auth").is_none() {
                return Err(anyhow::anyhow!("Login cancelled"));
            }
            if let Ok(url) = window.url() {
                let current = url.to_string();
                if current.starts_with(DESKTOP_REDIRECT) {
                    let _ = window.close();
                    return self.handle_desktop_callback(&current).await;
                }
            }
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
    }

    fn callback_params(callback_url: &str) -> HashMap<String, String> {
        callback_url
            .split('?')
            .nth(1)
            .unwrap_or("")
            .split('#')
            .next()
            .unwrap_or("")
            .split('&')
            .filter_map(|pair| {
                let mut parts = pair.splitn(2, '=');
                Some((
                    parts.next()?.to_string(),
                    percent_decode(parts.next().unwrap_or("")),
                ))
            })
            .collect()
    }

    async fn handle_desktop_callback(&self, callback_url: &str) -> Result<Account> {
        let params = Self::callback_params(callback_url);
        if let Some(err) = params.get("error") {
            let desc = params.get("error_description").cloned().unwrap_or_default();
            return Err(anyhow::anyhow!("Login failed: {} {}", err, desc));
        }
        let code = params
            .get("code")
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("No authorization code in response"))?;
        let ms_token = self.exchange_code_for_token(&code).await?;
        self.tokens_to_account(ms_token).await
    }

    async fn tokens_to_account(&self, ms_token: MicrosoftTokenResponse) -> Result<Account> {
        let xbl_token = self.authenticate_xbl(&ms_token.access_token).await?;
        let xsts_token = self.authenticate_xsts(&xbl_token.token).await?;
        let uhs = xbl_token
            .display_claims
            .xui
            .first()
            .map(|x| x.uhs.clone())
            .ok_or_else(|| anyhow::anyhow!("Missing Xbox user hash"))?;
        let mc_token = self
            .authenticate_minecraft(&uhs, &xsts_token.token)
            .await?;
        // Verify game ownership
        self.check_ownership(&mc_token.access_token).await?;
        let profile = self.get_minecraft_profile(&mc_token.access_token).await?;

        let now = Utc::now();
        Ok(Account {
            id: Uuid::new_v4().to_string(),
            username: profile.name.clone(),
            uuid: dash_uuid(&profile.id),
            access_token: Some(mc_token.access_token),
            refresh_token: Some(ms_token.refresh_token),
            token_expires_at: Some(now + chrono::Duration::seconds(mc_token.expires_in as i64)),
            account_type: AccountType::Microsoft,
            skin_url: profile.skins.first().map(|s| s.url.clone()),
            cape_url: profile.capes.first().map(|c| c.url.clone()),
            created_at: now,
            last_used: now,
        })
    }

    async fn exchange_code_for_token(&self, code: &str) -> Result<MicrosoftTokenResponse> {
        let params = [
            ("client_id", CLIENT_ID),
            ("code", code),
            ("grant_type", "authorization_code"),
            ("redirect_uri", DESKTOP_REDIRECT),
        ];
        let response = self.client.post(TOKEN_URL).form(&params).send().await?;
        if !response.status().is_success() {
            let error = response.text().await?;
            return Err(anyhow::anyhow!("Token exchange failed: {}", error));
        }
        Ok(response.json().await?)
    }

    async fn authenticate_xbl(&self, access_token: &str) -> Result<XboxLiveResponse> {
        let request = serde_json::json!({
            "Properties": {
                "AuthMethod": "RPS",
                "SiteName": "user.auth.xboxlive.com",
                "RpsTicket": format!("d={}", access_token)
            },
            "RelyingParty": "http://auth.xboxlive.com",
            "TokenType": "JWT"
        });
        let response = self.client.post(XBL_AUTH_URL).json(&request).send().await?;
        if !response.status().is_success() {
            let error = response.text().await?;
            return Err(anyhow::anyhow!("Xbox Live auth failed: {}", error));
        }
        Ok(response.json().await?)
    }

    async fn authenticate_xsts(&self, xbl_token: &str) -> Result<XstsResponse> {
        let request = serde_json::json!({
            "Properties": {
                "SandboxId": "RETAIL",
                "UserTokens": [xbl_token]
            },
            "RelyingParty": "rp://api.minecraftservices.com/",
            "TokenType": "JWT"
        });
        let response = self.client.post(XSTS_AUTH_URL).json(&request).send().await?;
        if !response.status().is_success() {
            let text = response.text().await?;
            if text.contains("2148916233") {
                return Err(anyhow::anyhow!(
                    "This Microsoft account is not linked to an Xbox account. Please create one at xbox.com"
                ));
            }
            if text.contains("2148916238") {
                return Err(anyhow::anyhow!(
                    "This account is underage or needs adult approval (XSTS 2148916238)"
                ));
            }
            return Err(anyhow::anyhow!("XSTS auth failed: {}", text));
        }
        Ok(response.json().await?)
    }

    async fn authenticate_minecraft(
        &self,
        uhs: &str,
        xsts_token: &str,
    ) -> Result<MinecraftAuthResponse> {
        let request = serde_json::json!({
            "identityToken": format!("XBL3.0 x={};{}", uhs, xsts_token)
        });
        let response = self
            .client
            .post(MINECRAFT_AUTH_URL)
            .json(&request)
            .send()
            .await?;
        if !response.status().is_success() {
            let error = response.text().await?;
            return Err(anyhow::anyhow!("Minecraft auth failed: {}", error));
        }
        Ok(response.json().await?)
    }

    async fn check_ownership(&self, mc_token: &str) -> Result<()> {
        let response = self
            .client
            .get(MINECRAFT_ENTITLEMENTS_URL)
            .bearer_auth(mc_token)
            .send()
            .await?;
        if !response.status().is_success() {
            let error = response.text().await?;
            return Err(anyhow::anyhow!("Entitlements check failed: {}", error));
        }
        let data: serde_json::Value = response.json().await?;
        let has_game = data["items"]
            .as_array()
            .map(|items| items.iter().any(|i| i["name"].as_str() == Some("game_minecraft")))
            .unwrap_or(false);
        if !has_game {
            return Err(anyhow::anyhow!(
                "This account does not own Minecraft. Buy it at minecraft.net"
            ));
        }
        Ok(())
    }

    async fn get_minecraft_profile(&self, access_token: &str) -> Result<MinecraftProfile> {
        let response = self
            .client
            .get(MINECRAFT_PROFILE_URL)
            .bearer_auth(access_token)
            .send()
            .await?;
        if response.status().as_u16() == 404 {
            return Err(anyhow::anyhow!(
                "No Minecraft profile found. Create one in the Minecraft launcher first"
            ));
        }
        if !response.status().is_success() {
            let error = response.text().await?;
            return Err(anyhow::anyhow!("Failed to get profile: {}", error));
        }
        Ok(response.json().await?)
    }

    pub async fn refresh_token(&self, account: &Account) -> Result<Account> {
        let refresh_token = account
            .refresh_token
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("No refresh token"))?;
        let params = [
            ("client_id", CLIENT_ID),
            ("refresh_token", refresh_token.as_str()),
            ("grant_type", "refresh_token"),
            ("redirect_uri", DESKTOP_REDIRECT),
        ];
        let response = self.client.post(TOKEN_URL).form(&params).send().await?;
        if !response.status().is_success() {
            let error = response.text().await?;
            return Err(anyhow::anyhow!("Token refresh failed: {}", error));
        }
        let ms_token: MicrosoftTokenResponse = response.json().await?;
        let mut updated = self.tokens_to_account(ms_token).await?;
        // keep stable id + created_at
        updated.id = account.id.clone();
        updated.created_at = account.created_at;
        updated.last_used = Utc::now();
        Ok(updated)
    }

    pub async fn validate_token(&self, account: &Account) -> Result<bool> {
        if account.account_type != AccountType::Microsoft {
            return Ok(true);
        }
        let token = match &account.access_token {
            Some(t) => t,
            None => return Ok(false),
        };
        if let Some(expires_at) = account.token_expires_at {
            if Utc::now() >= expires_at - chrono::Duration::minutes(5) {
                return Ok(false);
            }
        }
        // Lightweight server check
        let resp = self
            .client
            .get(MINECRAFT_PROFILE_URL)
            .bearer_auth(token)
            .send()
            .await;
        Ok(matches!(resp, Ok(r) if r.status().is_success()))
    }
}

fn percent_encode(s: &str) -> String {
    percent_encoding::utf8_percent_encode(s, percent_encoding::NON_ALPHANUMERIC).to_string()
}

fn percent_decode(s: &str) -> String {
    percent_encoding::percent_decode_str(s)
        .decode_utf8_lossy()
        .to_string()
}

fn dash_uuid(undashed: &str) -> String {
    if undashed.len() == 32 {
        format!(
            "{}-{}-{}-{}-{}",
            &undashed[0..8],
            &undashed[8..12],
            &undashed[12..16],
            &undashed[16..20],
            &undashed[20..32]
        )
    } else {
        undashed.to_string()
    }
}
