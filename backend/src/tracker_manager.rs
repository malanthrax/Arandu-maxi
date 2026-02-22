use crate::models::{TrackerConfig, TrackerModel, TrackerStats, WeeklyReport};
use rusqlite::{params, Connection};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;

pub struct TrackerManager {
    conn: Mutex<Connection>,
}

// Manual Debug implementation since Mutex<Connection> doesn't implement Debug
impl std::fmt::Debug for TrackerManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TrackerManager")
            .field("conn", &"<Mutex<Connection>>")
            .finish()
    }
}

impl TrackerManager {
    pub fn new(app_data_dir: PathBuf) -> Result<Self, String> {
        std::fs::create_dir_all(&app_data_dir)
            .map_err(|e| format!("Failed to create tracker directory: {}", e))?;

        let db_path = app_data_dir.join("tracker.db");
        let conn = Connection::open(&db_path)
            .map_err(|e| format!("Failed to open database: {}", e))?;

        let manager = Self {
            conn: Mutex::new(conn),
        };

        manager.init_db()?;
        
        Ok(manager)
    }

    fn init_db(&self) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS models (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                author TEXT,
                description TEXT,
                source TEXT DEFAULT 'huggingface',
                category TEXT,
                is_chinese INTEGER DEFAULT 0,
                is_gguf INTEGER DEFAULT 0,
                quantizations TEXT,
                backends TEXT,
                estimated_size_gb REAL,
                vram_requirement_gb REAL,
                context_length INTEGER,
                downloads INTEGER DEFAULT 0,
                likes INTEGER DEFAULT 0,
                last_updated TEXT,
                created_at TEXT
            )",
            [],
        ).map_err(|e| format!("Failed to create models table: {}", e))?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS tracker_config (
                key TEXT PRIMARY KEY,
                value TEXT
            )",
            [],
        ).map_err(|e| format!("Failed to create config table: {}", e))?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS weekly_reports (
                id TEXT PRIMARY KEY,
                generated_at TEXT,
                period_start TEXT,
                period_end TEXT,
                total_models INTEGER,
                new_models INTEGER,
                chinese_models INTEGER,
                gguf_models INTEGER,
                categories TEXT,
                top_downloads TEXT
            )",
            [],
        ).map_err(|e| format!("Failed to create weekly_reports table: {}", e))?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_category ON models(category)",
            [],
        ).map_err(|e| format!("Failed to create index: {}", e))?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_chinese ON models(is_chinese)",
            [],
        ).map_err(|e| format!("Failed to create index: {}", e))?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_gguf ON models(is_gguf)",
            [],
        ).map_err(|e| format!("Failed to create index: {}", e))?;

        Ok(())
    }

    pub fn clear_models(&self) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
        conn.execute("DELETE FROM models", [])
            .map_err(|e| format!("Failed to clear models: {}", e))?;
        Ok(())
    }

    pub fn save_models(&self, models: &[TrackerModel]) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;

        for model in models {
            let quantizations_json = serde_json::to_string(&model.quantizations)
                .unwrap_or_else(|_| "[]".to_string());
            let backends_json = serde_json::to_string(&model.backends)
                .unwrap_or_else(|_| "[]".to_string());

            conn.execute(
                "INSERT OR REPLACE INTO models 
                (id, name, author, description, source, category, is_chinese, is_gguf, 
                quantizations, backends, estimated_size_gb, vram_requirement_gb, 
                context_length, downloads, likes, last_updated, created_at)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)",
                params![
                    model.id,
                    model.name,
                    model.author,
                    model.description,
                    model.source,
                    model.category,
                    model.is_chinese as i32,
                    model.is_gguf as i32,
                    quantizations_json,
                    backends_json,
                    model.estimated_size_gb,
                    model.vram_requirement_gb,
                    model.context_length,
                    model.downloads,
                    model.likes,
                    model.last_updated,
                    model.created_at,
                ],
            ).map_err(|e| format!("Failed to save model: {}", e))?;
        }

        Ok(())
    }

    pub fn get_models(
        &self,
        vram_limit: Option<f64>,
        categories: Option<Vec<String>>,
        chinese_only: bool,
        gguf_only: bool,
        file_types: Option<Vec<String>>,
        quantizations: Option<Vec<String>>,
        search_query: Option<String>,
        sort_by: &str,
        sort_desc: bool,
    ) -> Result<Vec<TrackerModel>, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;

        let mut sql = String::from("SELECT * FROM models WHERE 1=1");
        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        // Debug logging
        eprintln!("DEBUG get_models filters:");
        eprintln!("  chinese_only: {}", chinese_only);
        eprintln!("  gguf_only: {}", gguf_only);
        eprintln!("  vram_limit: {:?}", vram_limit);
        eprintln!("  file_types: {:?}", file_types);
        eprintln!("  categories: {:?}", categories);

        if let Some(ref query) = search_query {
            if !query.is_empty() {
                sql.push_str(" AND (name LIKE ? OR description LIKE ? OR author LIKE ?)");
                let pattern = format!("%{}%", query);
                params_vec.push(Box::new(pattern.clone()));
                params_vec.push(Box::new(pattern.clone()));
                params_vec.push(Box::new(pattern));
            }
        }

        if chinese_only {
            sql.push_str(" AND is_chinese = 1");
        }

        if gguf_only {
            sql.push_str(" AND is_gguf = 1");
        }

        if let Some(ref cats) = categories {
            if !cats.is_empty() {
                // Use simple ? placeholders, not numbered ones
                let placeholders: Vec<String> = cats.iter()
                    .map(|_| "?".to_string())
                    .collect();
                sql.push_str(&format!(" AND category IN ({})", placeholders.join(",")));
                for cat in cats {
                    params_vec.push(Box::new(cat.clone()));
                }
            }
        }

        if let Some(limit) = vram_limit {
            // Use simple ? placeholder
            sql.push_str(" AND (vram_requirement_gb IS NULL OR vram_requirement_gb <= ?)");
            params_vec.push(Box::new(limit));
        }

        let order_col = match sort_by {
            "likes" => "likes",
            "date" => "last_updated",
            "name" => "name",
            "size" => "estimated_size_gb",
            _ => "downloads",
        };
        
        let order_dir = if sort_desc { "DESC" } else { "ASC" };
        sql.push_str(&format!(" ORDER BY {} {}", order_col, order_dir));

        let mut stmt = conn.prepare(&sql).map_err(|e| format!("Query error: {}", e))?;

        let params_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|p| p.as_ref()).collect();
        
        let models_iter = stmt.query_map(params_refs.as_slice(), |row| {
            let quant_json: String = row.get(8)?;
            let backends_json: String = row.get(9)?;

            Ok(TrackerModel {
                id: row.get(0)?,
                name: row.get(1)?,
                author: row.get(2)?,
                description: row.get(3)?,
                source: row.get(4)?,
                category: row.get(5)?,
                is_chinese: row.get::<_, i32>(6)? != 0,
                is_gguf: row.get::<_, i32>(7)? != 0,
                quantizations: serde_json::from_str(&quant_json).unwrap_or_default(),
                backends: serde_json::from_str(&backends_json).unwrap_or_default(),
                estimated_size_gb: row.get(10)?,
                vram_requirement_gb: row.get(11)?,
                context_length: row.get(12)?,
                downloads: row.get::<_, i64>(13)? as u64,
                likes: row.get::<_, i64>(14)? as u64,
                last_updated: row.get(15)?,
                created_at: row.get(16)?,
            })
        }).map_err(|e| format!("Query error: {}", e))?;

        let mut models = Vec::new();
        for model_result in models_iter {
            match model_result {
                Ok(model) => {
                    // File type filtering
                    if let Some(ref types) = file_types {
                        if !types.is_empty() {
                            let model_matches_type = types.iter().any(|t| {
                                let t_lower = t.to_lowercase();
                                match t_lower.as_str() {
                                    "gguf" => model.is_gguf,
                                    "mlx" => model.id.to_lowercase().contains("mlx") || model.name.to_lowercase().contains("mlx"),
                                    "safetensors" => model.id.to_lowercase().contains("safetensors") || model.name.to_lowercase().contains("safetensors"),
                                    "bin" => model.id.to_lowercase().contains(".bin") || model.name.to_lowercase().contains(".bin"),
                                    "pt" => model.id.to_lowercase().contains(".pt") || model.name.to_lowercase().contains(".pt") || 
                                             model.id.to_lowercase().contains("pytorch") || model.name.to_lowercase().contains("pytorch"),
                                    _ => false
                                }
                            });
                            if !model_matches_type {
                                continue;
                            }
                        }
                    }

                    if let Some(ref quants) = quantizations {
                        if !quants.is_empty() {
                            let model_has_quant = model.quantizations.iter()
                                .any(|q| quants.iter().any(|uq| uq.to_lowercase() == q.to_lowercase()));
                            if !model_has_quant {
                                continue;
                            }
                        }
                    }

                    models.push(model);
                }
                Err(e) => {
                    eprintln!("Error loading model: {}", e);
                }
            }
        }

        // Debug logging
        eprintln!("DEBUG SQL query: {}", sql);
        eprintln!("DEBUG params count: {}", params_vec.len());
        eprintln!("DEBUG returned models: {}", models.len());

        Ok(models)
    }

    pub fn get_stats(&self) -> Result<TrackerStats, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;

        let total: u32 = conn.query_row(
            "SELECT COUNT(*) FROM models",
            [],
            |row| row.get(0),
        ).unwrap_or(0);

        let chinese: u32 = conn.query_row(
            "SELECT COUNT(*) FROM models WHERE is_chinese = 1",
            [],
            |row| row.get(0),
        ).unwrap_or(0);

        let gguf: u32 = conn.query_row(
            "SELECT COUNT(*) FROM models WHERE is_gguf = 1",
            [],
            |row| row.get(0),
        ).unwrap_or(0);

        let mut stmt = conn.prepare(
            "SELECT category, COUNT(*) as count FROM models GROUP BY category"
        ).map_err(|e| format!("Query error: {}", e))?;

        let categories_iter = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, u32>(1)?))
        }).map_err(|e| format!("Query error: {}", e))?;

        let mut categories = HashMap::new();
        for cat_result in categories_iter {
            if let Ok((cat, count)) = cat_result {
                categories.insert(cat, count);
            }
        }

        Ok(TrackerStats {
            total_models: total,
            chinese_models: chinese,
            gguf_models: gguf,
            categories,
        })
    }

    pub fn get_config(&self) -> Result<TrackerConfig, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;

        let result: Result<String, _> = conn.query_row(
            "SELECT value FROM tracker_config WHERE key = 'config'",
            [],
            |row| row.get(0),
        );

        match result {
            Ok(json) => {
                serde_json::from_str(&json).map_err(|e| format!("Config parse error: {}", e))
            }
            Err(_) => Ok(TrackerConfig::default()),
        }
    }

    pub fn save_config(&self, config: &TrackerConfig) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;

        let json = serde_json::to_string(config)
            .map_err(|e| format!("Config serialize error: {}", e))?;

        conn.execute(
            "INSERT OR REPLACE INTO tracker_config (key, value) VALUES ('config', ?1)",
            params![json],
        ).map_err(|e| format!("Failed to save config: {}", e))?;

        Ok(())
    }

    pub fn export_json(&self) -> Result<String, String> {
        let models = self.get_models(
            None, None, false, false, None, None, None, "downloads", true
        )?;
        
        serde_json::to_string_pretty(&models)
            .map_err(|e| format!("Export error: {}", e))
    }

    pub fn get_weekly_reports(&self, limit: u32) -> Result<Vec<WeeklyReport>, String> {
        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;

        let mut stmt = conn.prepare(
            "SELECT id, generated_at, period_start, period_end, total_models, new_models, 
             chinese_models, gguf_models, categories, top_downloads 
             FROM weekly_reports ORDER BY generated_at DESC LIMIT ?1"
        ).map_err(|e| format!("Query error: {}", e))?;

        let reports = stmt.query_map(params![limit], |row| {
            let categories_json: String = row.get(8)?;
            let top_downloads_json: String = row.get(9)?;

            Ok(WeeklyReport {
                id: row.get(0)?,
                generated_at: row.get(1)?,
                period_start: row.get(2)?,
                period_end: row.get(3)?,
                total_models: row.get(4)?,
                new_models: row.get(5)?,
                chinese_models: row.get(6)?,
                gguf_models: row.get(7)?,
                categories: serde_json::from_str(&categories_json).unwrap_or_default(),
                top_downloads: serde_json::from_str(&top_downloads_json).unwrap_or_default(),
            })
        }).map_err(|e| format!("Query error: {}", e))?;

        let mut result = Vec::new();
        for report in reports {
            if let Ok(r) = report {
                result.push(r);
            }
        }

        Ok(result)
    }

    pub fn generate_weekly_report(&self) -> Result<WeeklyReport, String> {
        let now = chrono::Utc::now();
        let week_ago = now - chrono::Duration::days(7);

        let conn = self.conn.lock().map_err(|e| format!("Lock error: {}", e))?;

        // Get all models for stats
        let total: u32 = conn.query_row(
            "SELECT COUNT(*) FROM models", [], |row| row.get(0)
        ).unwrap_or(0);

        let chinese: u32 = conn.query_row(
            "SELECT COUNT(*) FROM models WHERE is_chinese = 1", [], |row| row.get(0)
        ).unwrap_or(0);

        let gguf: u32 = conn.query_row(
            "SELECT COUNT(*) FROM models WHERE is_gguf = 1", [], |row| row.get(0)
        ).unwrap_or(0);

        // Get categories
        let mut stmt = conn.prepare(
            "SELECT category, COUNT(*) as count FROM models GROUP BY category"
        ).map_err(|e| format!("Query error: {}", e))?;

        let categories_iter = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, u32>(1)?))
        }).map_err(|e| format!("Query error: {}", e))?;

        let mut categories = HashMap::new();
        for cat_result in categories_iter {
            if let Ok((cat, count)) = cat_result {
                categories.insert(cat, count);
            }
        }

        // Get top downloads
        let mut top_stmt = conn.prepare(
            "SELECT id, name, author, description, source, category, is_chinese, is_gguf,
             quantizations, backends, estimated_size_gb, vram_requirement_gb, context_length,
             downloads, likes, last_updated, created_at
             FROM models ORDER BY downloads DESC LIMIT 10"
        ).map_err(|e| format!("Query error: {}", e))?;

        let top_iter = top_stmt.query_map([], |row| {
            let quant_json: String = row.get(8)?;
            let back_json: String = row.get(9)?;

            Ok(TrackerModel {
                id: row.get(0)?,
                name: row.get(1)?,
                author: row.get(2)?,
                description: row.get(3)?,
                source: row.get(4)?,
                category: row.get(5)?,
                is_chinese: row.get::<_, i32>(6)? != 0,
                is_gguf: row.get::<_, i32>(7)? != 0,
                quantizations: serde_json::from_str(&quant_json).unwrap_or_default(),
                backends: serde_json::from_str(&back_json).unwrap_or_default(),
                estimated_size_gb: row.get(10)?,
                vram_requirement_gb: row.get(11)?,
                context_length: row.get(12)?,
                downloads: row.get::<_, i64>(13)? as u64,
                likes: row.get::<_, i64>(14)? as u64,
                last_updated: row.get(15)?,
                created_at: row.get(16)?,
            })
        }).map_err(|e| format!("Query error: {}", e))?;

        let mut top_downloads = Vec::new();
        for model in top_iter {
            if let Ok(m) = model {
                top_downloads.push(m);
            }
        }

        let report = WeeklyReport {
            id: uuid::Uuid::new_v4().to_string(),
            generated_at: now.to_rfc3339(),
            period_start: week_ago.to_rfc3339(),
            period_end: now.to_rfc3339(),
            total_models: total,
            new_models: 0, // Would need previous snapshot to calculate
            chinese_models: chinese,
            gguf_models: gguf,
            categories,
            top_downloads,
        };

        // Save report
        let categories_json = serde_json::to_string(&report.categories)
            .unwrap_or_else(|_| "{}".to_string());
        let top_json = serde_json::to_string(&report.top_downloads)
            .unwrap_or_else(|_| "[]".to_string());

        conn.execute(
            "INSERT INTO weekly_reports 
            (id, generated_at, period_start, period_end, total_models, new_models, 
             chinese_models, gguf_models, categories, top_downloads)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                report.id,
                report.generated_at,
                report.period_start,
                report.period_end,
                report.total_models,
                report.new_models,
                report.chinese_models,
                report.gguf_models,
                categories_json,
                top_json,
            ],
        ).map_err(|e| format!("Failed to save report: {}", e))?;

        // Keep only last 4 reports
        conn.execute(
            "DELETE FROM weekly_reports WHERE id NOT IN 
             (SELECT id FROM weekly_reports ORDER BY generated_at DESC LIMIT 4)",
            [],
        ).map_err(|e| format!("Failed to cleanup old reports: {}", e))?;

        Ok(report)
    }
}
