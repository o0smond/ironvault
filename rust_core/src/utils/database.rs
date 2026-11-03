/*
Made by: Mathew Dusome
Jul 31 2026
Turso (libSQL) database module for Rust
*/

use serde::{Deserialize, Serialize};
#[cfg(not(target_arch = "wasm32"))]
use ureq;

// Helper function for serde to skip serializing id when it's 0
fn is_zero(num: &i32) -> bool {
    *num == 0
}

// URL of your Cloudflare Worker backend
pub const WORKER_URL: &str = "https://ironvault-bridge.meeses777.workers.dev";

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DatabaseTable {
    #[serde(default, skip_serializing_if = "is_zero")]
    pub id: i32,
    pub username: String,
    pub ciphertext: String,
    pub kdf_salt: String,
    pub nonce: String,
    pub password_hint: String
}

impl DatabaseTable {
    pub fn new(id: i32, username: String, ciphertext: String, kdf_salt: String, nonce: String, password_hint: String) -> Self{
        DatabaseTable { id, username, ciphertext, kdf_salt, nonce, password_hint }
    }
}

#[allow(unused)]
pub fn create_database_client() -> DatabaseClient {
    DatabaseClient::new(WORKER_URL.to_string())
}

pub struct DatabaseClient {
    worker_url: String,
}

impl DatabaseClient {
        #[allow(unused)]    
        pub async fn insert_record<T: Serialize>(&self, table: &str, record: &T) -> Result<i64, Box<dyn std::error::Error>> {
            let payload = serde_json::json!({
                "action": "insert",
                "table": table,
                "record": record
            });
            let resp = self.send_request(&payload).await?;
            Ok(resp["id"].as_i64().unwrap_or(0))
        }

        #[allow(unused)]
        pub async fn update_record_by_struct<T: Serialize>(&self, table: &str, record: &T) -> Result<i64, Box<dyn std::error::Error>> {
            let payload = serde_json::json!({
                "action": "update",
                "table": table,
                "record": record
            });
            let resp = self.send_request(&payload).await?;
            Ok(resp["updated"].as_i64().unwrap_or(0))
        }

        #[allow(unused)]
        pub async fn update_record_by_id(&self, table: &str, id: i64, column: &str, value: &serde_json::Value) -> Result<i64, Box<dyn std::error::Error>> {
            let payload = serde_json::json!({
                "action": "update_by_column",
                "table": table,
                "id": id,
                "column": column,
                "value": value
            });
            let resp = self.send_request(&payload).await?;
            Ok(resp["updated"].as_i64().unwrap_or(0))
        }

        #[allow(unused)]
        pub async fn delete_record_by_id(&self, table: &str, id: i64) -> Result<i64, Box<dyn std::error::Error>> {
            let payload = serde_json::json!({
                "action": "delete",
                "table": table,
                "id": id
            });
            let resp = self.send_request(&payload).await?;
            Ok(resp["deleted"].as_i64().unwrap_or(0))
        }
        
        #[allow(unused)]
        async fn send_request(&self, payload: &serde_json::Value) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
            let body = payload.to_string();
            
            // 1. Native / IDE checking block
            #[cfg(not(target_arch = "wasm32"))]
            {
                let url = &self.worker_url;
                let response = ureq::post(url)
                    .set("Content-Type", "application/json")
                    .send_string(&body);
                let text = match response {
                    Ok(resp) => resp.into_string()?,
                    Err(ureq::Error::Status(code, resp)) => {
                        let err_body = resp.into_string().unwrap_or_else(|_| "Could not read error body".to_string());
                        return Err(format!("HTTP {} error: {}", code, err_body).into());
                    }
                    Err(e) => return Err(e.into()),
                };
                let json: serde_json::Value = serde_json::from_str(&text)?;
                return Ok(json);
            }
            
            // 2. WebAssembly block (used by wasm-pack)
            #[cfg(target_arch = "wasm32")]
            {
                let response = gloo_net::http::Request::post(&self.worker_url)
                    .json(payload)
                    .map_err(|e| format!("JSON payload error: {}", e))?
                    .send()
                    .await
                    .map_err(|e| format!("Network error: {}", e))?;

                if !response.ok() {
                    let err_body = response.text().await.unwrap_or_default();
                    return Err(format!("HTTP {} error: {}", response.status(), err_body).into());
                }

                let json: serde_json::Value = response.json().await
                    .map_err(|e| format!("Failed to parse JSON response: {}", e))?;
                
                return Ok(json);
            }
        }
    pub fn new(worker_url: String) -> Self {
        Self { worker_url }
    }


   
    #[allow(unused)]
    pub async fn fetch_table<T: for<'de> Deserialize<'de>>(&self, table: &str) -> Result<Vec<T>, Box<dyn std::error::Error>> {
        let payload = serde_json::json!({
            "action": "fetch",
            "table": table
        });
        let resp = self.send_request(&payload).await?;
        let records = resp["records"].as_array().cloned().unwrap_or_default();
        let mut result = Vec::new();
        for record in records {
            result.push(serde_json::from_value(record)?);
        }
        Ok(result)
    }
    #[allow(unused)]
    pub async fn fetch_record_by_id<T: for<'de> Deserialize<'de>>(&self, table: &str, id: i64) -> Result<Option<T>, Box<dyn std::error::Error>> {
        let payload = serde_json::json!({
            "action": "fetch_by_id",
            "table": table,
            "id": id
        });
        let resp = self.send_request(&payload).await?;
        let record = resp.get("record").cloned();
        match record {
            Some(val) if !val.is_null() => Ok(Some(serde_json::from_value(val)?)),
            _ => Ok(None)
        }
    }

    #[allow(unused)]
    pub async fn fetch_id_by_user<T: for<'de> Deserialize<'de>>(&self, table: &str, username: &str) -> Result<Option<T>, Box<dyn std::error::Error>> {
        let payload = serde_json::json!({
            "action": "fetch_by_user",
            "table": table,
            "username": username
        });
        let resp = match self.send_request(&payload).await {
            Ok(val) => val,
            // The worker answers 404 when the username isn't in the table. Anything else is a real failure.
            Err(e) if e.to_string().starts_with("HTTP 404") => return Ok(None),
            Err(e) => return Err(e),
        };

        let id = resp.get("id").cloned();
        match id {
            Some(val) if !val.is_null() => Ok(Some(serde_json::from_value(val)?)),
            _ => Ok(None),
        }
    }
}
