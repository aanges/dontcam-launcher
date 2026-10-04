use super::*;
use anyhow::Result;
use chrono::Utc;
use uuid::Uuid;

/// Offline auth uses Java-correct UUID v3: MD5("OfflinePlayer:<name>")
pub struct OfflineAuth;

impl OfflineAuth {
    pub fn new() -> Self {
        Self
    }

    pub async fn authenticate(&self, username: String) -> Result<Account> {
        let username = username.trim().to_string();
        if username.is_empty() {
            return Err(anyhow::anyhow!("Username cannot be empty"));
        }
        if username.len() > 16 {
            return Err(anyhow::anyhow!("Username too long (max 16 characters)"));
        }
        if !username
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' )
        {
            return Err(anyhow::anyhow!(
                "Username may only contain letters, numbers and underscore"
            ));
        }

        let uuid = generate_offline_uuid(&username);
        let now = Utc::now();

        Ok(Account {
            id: Uuid::new_v4().to_string(),
            username: username.clone(),
            uuid,
            access_token: None,
            refresh_token: None,
            token_expires_at: None,
            account_type: AccountType::Offline,
            skin_url: Some(format!("https://minotar.net/helm/{}//100.png", username)),
            cape_url: None,
            created_at: now,
            last_used: now,
        })
    }
}

pub fn generate_offline_uuid(username: &str) -> String {
    let input = format!("OfflinePlayer:{}", username);
    let digest = md5::compute(input.as_bytes());
    let mut bytes = digest.0;
    // UUID v3: version 3, variant RFC4122
    bytes[6] = (bytes[6] & 0x0f) | 0x30;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    // Minecraft uses undashed UUID in most places, but keep dashed for storage
    Uuid::from_bytes(bytes).to_string()
}

#[cfg(test)]
mod tests {
    use super::generate_offline_uuid;
    #[test]
    fn known_offline_uuid() {
        // UUID v3 = MD5("OfflinePlayer:Notch") with version/variant bits.
        // (069a79f4-... is Notch's *premium* UUID, not the offline one.)
        assert_eq!(
            generate_offline_uuid("Notch"),
            "b50ad385-829d-3141-a216-7e7d7539ba7f"
        );
    }
}
