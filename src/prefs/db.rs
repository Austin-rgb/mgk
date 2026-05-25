use anyhow::Result;
use moka::future::Cache;
use rand::RngExt;
use serde::Deserialize;
use sqlx::{Pool, Sqlite};
use validator::Validate;

fn gen_otp() -> u32 {
    let mut rng = rand::rng();
    rng.random_range(100000..999999)
}

// Models

#[derive(Deserialize, Validate, Clone)]
pub struct Preference {
    #[validate(length(max = 64))]
    pub subject: String,
    #[validate(length(max = 64))]
    pub address: String,
}

#[derive(Deserialize, Validate)]
pub struct Token {
    #[validate(range(min = 100000, max = 999999))]
    pub token: u32,
}

// ---------------------------------------------------------------------------
// Preferences
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct Preferences {
    db: Pool<Sqlite>,
    cache: Cache<(String, String), String>, // (user, subject) -> Channel
    pending: Cache<(String, u32), (String, String)>,
}

impl Preferences {
    pub fn new(db: Pool<Sqlite>) -> Self {
        Self {
            db,
            cache: Cache::new(1000),
            pending: Cache::new(100),
        }
    }

    pub async fn confirm(&self, user: &str, otp: &Token) -> Result<()> {
        if let Err(e) = otp.validate() {
            return Err(anyhow::anyhow!("invalid token: {e}"));
        }
        let (subject, channel) = match self.pending.remove(&(user.to_string(), otp.token)).await {
            Some(r) => r,
            None => return Err(anyhow::anyhow!("Token not found")),
        };
        sqlx::query!(
            "INSERT INTO preferences (user, subject, address)
             VALUES (?, ?, ?)
             ON CONFLICT(address, subject)
             DO UPDATE SET address = excluded.address",
        
        user,
        subject,
        channel)
        .execute(&self.db)
        .await?;

        self.cache
            .insert((user.to_string(), subject.to_string()), channel)
            .await;

        Ok(())
    }

    pub async fn get(&self, user: &str, subject: &str) -> Result<Option<String>> {
        let key = (user.to_string(), subject.to_string());

        if let Some(cached) = self.cache.get(&key).await {
            return Ok(Some(cached));
        }

        let result = sqlx::query_scalar::<_, String>(
            "SELECT address FROM preferences WHERE user = ? AND subject = ?",
        )
        .bind(user)
        .bind(subject)
        .fetch_optional(&self.db)
        .await?;

        if let Some(channel) = result {
            self.cache.insert(key, channel.clone()).await;
            return Ok(Some(channel));
        }
        Ok(None)
    }

    pub async fn set(&self, user: &str, pref: Preference) -> Result<u32> {
        if let Err(e) = pref.validate() {
            return Err(anyhow::anyhow!("Invalid data: {e}"));
        }
        let otp = gen_otp();
        self.pending
            .insert((user.into(), otp), (pref.subject, pref.address))
            .await;
        Ok(otp)
    }
}
