//! 数据库模块
//! 管理 SQLite 数据库连接和操作

use crate::error::{AppError, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::{sqlite::SqlitePoolOptions, Pool, Row, Sqlite};
use std::path::PathBuf;
use tracing::{debug, error, info};

/// 数据库管理器
pub struct Database {
    pool: Pool<Sqlite>,
}

/// 翻译记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationRecord {
    pub id: i64,
    pub original_text: String,
    pub translated_text: String,
    pub source_lang: Option<String>,
    pub target_lang: String,
    pub mode: String,
    pub timestamp: i64,
}

/// 查询历史记录的结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryResult {
    pub records: Vec<TranslationRecord>,
    pub total: i64,
}

impl Database {
    /// 创建数据库连接
    pub async fn new() -> Result<Self> {
        let db_path = Self::get_db_path()?;
        
        // 确保目录存在
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let db_url = format!("sqlite:{}?mode=rwc", db_path.display());
        debug!("Connecting to database: {}", db_url);

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(&db_url)
            .await?;

        let db = Self { pool };
        db.run_migrations().await?;
        
        info!("Database initialized successfully");
        Ok(db)
    }

    /// 获取数据库文件路径
    fn get_db_path() -> Result<PathBuf> {
        let data_dir = dirs::data_dir()
            .ok_or_else(|| AppError::Config("无法获取数据目录".to_string()))?;
        Ok(data_dir.join("AITyping").join("aityping.db"))
    }

    /// 运行数据库迁移
    async fn run_migrations(&self) -> Result<()> {
        debug!("Running database migrations...");

        // 创建翻译记录表
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS translations (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                original_text TEXT NOT NULL,
                translated_text TEXT NOT NULL,
                source_lang TEXT,
                target_lang TEXT NOT NULL,
                mode TEXT NOT NULL,
                timestamp INTEGER NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // 创建索引
        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_translations_timestamp ON translations(timestamp DESC)",
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_translations_lang ON translations(target_lang, source_lang)",
        )
        .execute(&self.pool)
        .await?;

        // 创建性能指标表
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS metrics (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp INTEGER NOT NULL,
                operation_type TEXT NOT NULL,
                duration_ms INTEGER NOT NULL,
                success INTEGER NOT NULL,
                error_type TEXT,
                char_count INTEGER NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_metrics_timestamp ON metrics(timestamp DESC)",
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_metrics_operation ON metrics(operation_type)",
        )
        .execute(&self.pool)
        .await?;

        debug!("Database migrations completed");
        Ok(())
    }

    /// 插入翻译记录
    pub async fn insert_translation(
        &self,
        original_text: &str,
        translated_text: &str,
        source_lang: Option<&str>,
        target_lang: &str,
        mode: &str,
    ) -> Result<i64> {
        let timestamp = Utc::now().timestamp();

        let result = sqlx::query(
            r#"
            INSERT INTO translations (original_text, translated_text, source_lang, target_lang, mode, timestamp)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(original_text)
        .bind(translated_text)
        .bind(source_lang)
        .bind(target_lang)
        .bind(mode)
        .bind(timestamp)
        .execute(&self.pool)
        .await?;

        Ok(result.last_insert_rowid())
    }

    /// 查询翻译历史
    pub async fn get_history(
        &self,
        page: i64,
        page_size: i64,
        search: Option<&str>,
        mode: Option<&str>,
    ) -> Result<HistoryResult> {
        let offset = (page - 1) * page_size;

        // 构建查询条件
        let mut conditions = Vec::new();
        if search.is_some() {
            conditions.push("(original_text LIKE ? OR translated_text LIKE ?)");
        }
        if mode.is_some() {
            conditions.push("mode = ?");
        }

        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", conditions.join(" AND "))
        };

        // 查询总数
        let count_query = format!("SELECT COUNT(*) as count FROM translations {}", where_clause);
        let mut count_builder = sqlx::query(&count_query);
        
        if let Some(s) = search {
            let pattern = format!("%{}%", s);
            count_builder = count_builder.bind(pattern.clone()).bind(pattern);
        }
        if let Some(m) = mode {
            count_builder = count_builder.bind(m);
        }

        let total: i64 = count_builder
            .fetch_one(&self.pool)
            .await?
            .get("count");

        // 查询记录
        let data_query = format!(
            "SELECT * FROM translations {} ORDER BY timestamp DESC LIMIT ? OFFSET ?",
            where_clause
        );
        let mut data_builder = sqlx::query(&data_query);
        
        if let Some(s) = search {
            let pattern = format!("%{}%", s);
            data_builder = data_builder.bind(pattern.clone()).bind(pattern);
        }
        if let Some(m) = mode {
            data_builder = data_builder.bind(m);
        }
        
        data_builder = data_builder.bind(page_size).bind(offset);

        let rows = data_builder.fetch_all(&self.pool).await?;

        let records: Vec<TranslationRecord> = rows
            .iter()
            .map(|row| TranslationRecord {
                id: row.get("id"),
                original_text: row.get("original_text"),
                translated_text: row.get("translated_text"),
                source_lang: row.get("source_lang"),
                target_lang: row.get("target_lang"),
                mode: row.get("mode"),
                timestamp: row.get("timestamp"),
            })
            .collect();

        Ok(HistoryResult { records, total })
    }

    /// 清理超出限制的历史记录
    pub async fn cleanup_history(&self, limit: usize) -> Result<u64> {
        let result = sqlx::query(
            r#"
            DELETE FROM translations 
            WHERE id NOT IN (
                SELECT id FROM translations 
                ORDER BY timestamp DESC 
                LIMIT ?
            )
            "#,
        )
        .bind(limit as i64)
        .execute(&self.pool)
        .await?;

        let deleted = result.rows_affected();
        if deleted > 0 {
            debug!("Cleaned up {} old translation records", deleted);
        }
        Ok(deleted)
    }

    /// 记录性能指标
    pub async fn record_metric(
        &self,
        operation_type: &str,
        duration_ms: i64,
        success: bool,
        error_type: Option<&str>,
        char_count: i64,
    ) -> Result<()> {
        let timestamp = Utc::now().timestamp();

        sqlx::query(
            r#"
            INSERT INTO metrics (timestamp, operation_type, duration_ms, success, error_type, char_count)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(timestamp)
        .bind(operation_type)
        .bind(duration_ms)
        .bind(success)
        .bind(error_type)
        .bind(char_count)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// 获取性能统计
    pub async fn get_performance_stats(&self, period: &str) -> Result<PerformanceStats> {
        let since = match period {
            "hour" => Utc::now().timestamp() - 3600,
            "day" => Utc::now().timestamp() - 86400,
            "week" => Utc::now().timestamp() - 604800,
            _ => Utc::now().timestamp() - 86400,
        };

        // 基本统计
        let stats_row = sqlx::query(
            r#"
            SELECT 
                COUNT(*) as total,
                SUM(CASE WHEN success = 1 THEN 1 ELSE 0 END) as successful,
                SUM(CASE WHEN success = 0 THEN 1 ELSE 0 END) as failed,
                AVG(CASE WHEN success = 1 THEN duration_ms ELSE NULL END) as avg_duration,
                MIN(CASE WHEN success = 1 THEN duration_ms ELSE NULL END) as min_duration,
                MAX(CASE WHEN success = 1 THEN duration_ms ELSE NULL END) as max_duration,
                SUM(char_count) as total_chars,
                SUM(CASE WHEN operation_type = 'selected' THEN 1 ELSE 0 END) as selected_count,
                SUM(CASE WHEN operation_type = 'full' THEN 1 ELSE 0 END) as full_count
            FROM metrics
            WHERE timestamp > ?
            "#,
        )
        .bind(since)
        .fetch_one(&self.pool)
        .await?;

        // 错误分布
        let error_rows = sqlx::query(
            r#"
            SELECT error_type, COUNT(*) as count
            FROM metrics
            WHERE timestamp > ? AND success = 0 AND error_type IS NOT NULL
            GROUP BY error_type
            "#,
        )
        .bind(since)
        .fetch_all(&self.pool)
        .await?;

        let error_distribution: Vec<ErrorDistribution> = error_rows
            .iter()
            .map(|row| ErrorDistribution {
                error_type: row.get("error_type"),
                count: row.get("count"),
            })
            .collect();

        Ok(PerformanceStats {
            total_translations: stats_row.get::<i64, _>("total") as u64,
            successful_translations: stats_row.get::<i64, _>("successful") as u64,
            failed_translations: stats_row.get::<i64, _>("failed") as u64,
            avg_duration_ms: stats_row.get::<Option<f64>, _>("avg_duration").unwrap_or(0.0),
            min_duration_ms: stats_row.get::<Option<i64>, _>("min_duration").unwrap_or(0) as u64,
            max_duration_ms: stats_row.get::<Option<i64>, _>("max_duration").unwrap_or(0) as u64,
            total_chars_translated: stats_row.get::<Option<i64>, _>("total_chars").unwrap_or(0) as u64,
            selected_mode_count: stats_row.get::<i64, _>("selected_count") as u64,
            full_mode_count: stats_row.get::<i64, _>("full_count") as u64,
            error_distribution,
            hourly_data: Vec::new(), // TODO: 实现按小时统计
        })
    }

    /// 清理旧的性能指标（保留 90 天）
    pub async fn cleanup_metrics(&self) -> Result<u64> {
        let cutoff = Utc::now().timestamp() - (90 * 24 * 3600);
        
        let result = sqlx::query("DELETE FROM metrics WHERE timestamp < ?")
            .bind(cutoff)
            .execute(&self.pool)
            .await?;

        let deleted = result.rows_affected();
        if deleted > 0 {
            debug!("Cleaned up {} old metric records", deleted);
        }
        Ok(deleted)
    }
}

/// 性能统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceStats {
    pub total_translations: u64,
    pub successful_translations: u64,
    pub failed_translations: u64,
    pub avg_duration_ms: f64,
    pub min_duration_ms: u64,
    pub max_duration_ms: u64,
    pub total_chars_translated: u64,
    pub selected_mode_count: u64,
    pub full_mode_count: u64,
    pub error_distribution: Vec<ErrorDistribution>,
    pub hourly_data: Vec<HourlyData>,
}

/// 错误分布
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorDistribution {
    pub error_type: String,
    pub count: i64,
}

/// 按小时统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HourlyData {
    pub hour: i32,
    pub avg_duration: f64,
    pub count: i64,
}
