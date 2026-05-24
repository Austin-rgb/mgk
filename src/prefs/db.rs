use anyhow::Result;
use moka::future::Cache;
use sqlx::{Pool, Sqlite};

use rand::RngExt;

fn gen_otp() -> u32 {
    let mut rng = rand::rng();
    rng.random_range(100000..999999)
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

    pub async fn confirm(&self, user: &str, otp: u32) -> Result<()> {
        let (subject, channel) = match self.pending.remove(&(user.to_string(), otp)).await {
            Some(r) => r,
            None => return Err(anyhow::anyhow!("Token not found")),
        };
        sqlx::query(
            "INSERT INTO preferences (user, subject, channel)
             VALUES (?, ?, ?)
             ON CONFLICT(user, subject)
             DO UPDATE SET channel = excluded.channel",
        )
        .bind(user)
        .bind(subject.clone())
        .bind(channel.clone())
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
            "SELECT channel FROM preferences WHERE user = ? AND subject = ?",
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

    pub async fn set(&self, user: &str, subject: &str, addr: &str) -> Result<u32> {
        let otp = gen_otp();
        self.pending
            .insert((user.into(), otp), (subject.into(), addr.into()))
            .await;
        Ok(otp)
    }
}
