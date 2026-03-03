use anyhow::Result;

use super::db::DB;

#[derive(Debug, Clone)]
pub struct ImageEntity {
    pub id: i32,
    pub file_index: i32,
    pub hash: String,
    pub url: String,
}

impl ImageEntity {
    pub async fn create(file_index: i32, hash: String, url: &str) -> Result<i32> {
        let pool = DB.get().unwrap();
        let rec = sqlx::query!(
            "INSERT OR IGNORE INTO images (file_index, hash, url) VALUES (?, ?, ?)",
            file_index,
            hash,
            url
        )
        .execute(pool)
        .await?;
        Ok(rec.last_insert_rowid() as i32)
    }

    pub async fn get_by_hash(hash: String) -> Result<Option<Self>> {
        let pool = DB.get().unwrap();
        let rec = sqlx::query_as!(Self, "SELECT * FROM images WHERE hash = ?", hash)
            .fetch_optional(pool)
            .await?;
        Ok(rec)
    }

    pub async fn get_by_gallery_id(id: i32) -> Result<Vec<Self>> {
        let pool = DB.get().unwrap();
        let rec = sqlx::query_as!(
            Self,
            r#"
            SELECT T1.* FROM images AS T1
            LEFT JOIN pages AS T2 ON T1.id = T2.image_id
            WHERE T2.gallery_id = ?
            ORDER BY T2.page
            "#,
            id
        )
        .fetch_all(pool)
        .await?;
        Ok(rec)
    }

    // ========================================================================
    // 🔥 第一階段新增功能 (Stats)
    // ========================================================================

    /// 統計圖片總數
    pub async fn count() -> Result<i64> {
        let pool = DB.get().ok_or(anyhow::anyhow!("資料庫未連接"))?;
        let rec = sqlx::query!("SELECT COUNT(*) as count FROM images")
            .fetch_one(pool)
            .await?;
        Ok(rec.count as i64)
    }
}
