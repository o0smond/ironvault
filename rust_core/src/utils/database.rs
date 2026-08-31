/*
Made by: Mathew Dusome
Jul 31 2026
Turso (libSQL) database module for Rust
*/

use serde::{Deserialize, Serialize};
#[cfg(not(target_arch = "wasm32"))]
use ureq;
#[cfg(target_arch = "wasm32")]
use macroquad::prelude::next_frame;

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
            
            #[cfg(target_arch = "wasm32")]
            {
                #[link(wasm_import_module = "env")]
                extern "C" {
                    fn mq_db_query(ptr: *const u8, len: usize, url_ptr: *const u8, url_len: usize);
                    fn mq_db_query_result_len() -> usize;
                    fn mq_db_query_fill_result(ptr: *mut u8);
                    fn mq_db_query_clear_result();
                }
                
                let url_bytes = self.worker_url.as_bytes();
                let json_bytes = body.as_bytes();
                // Call JS: mq_db_query(ptr, len, url_ptr, url_len)
                unsafe {
                    mq_db_query(
                        json_bytes.as_ptr(),
                        json_bytes.len(),
                        url_bytes.as_ptr(),
                        url_bytes.len(),
                    );
                }
                let mut tries = 0;
                let max_tries = 100;
                let mut result_len = 0;
                while tries < max_tries {
                    result_len = unsafe { mq_db_query_result_len() };
                    if result_len > 0 {
                        break;
                    }
                    tries += 1;
                    next_frame().await;
                }
                if result_len == 0 {
                    return Err("No result from JS db_query (timeout or JS error)".into());
                }
                let mut buf = vec![0u8; result_len];
                unsafe {
                    mq_db_query_fill_result(buf.as_mut_ptr());
                    mq_db_query_clear_result();
                }
                let text = String::from_utf8(buf).map_err(|e| format!("UTF-8 error: {}", e))?;
                let json: serde_json::Value = serde_json::from_str(&text)?;
                return Ok(json);
            }
            
            #[cfg(not(any(target_arch = "wasm32", not(target_arch = "wasm32"))))]
            {
                unreachable!("This should never be reached");
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
            Err(_) => return Ok(None), // Handles 404 / user not found responses
        };

        let id = resp.get("id").cloned();
        match id {
            Some(val) if !val.is_null() => Ok(Some(serde_json::from_value(val)?)),
            _ => Ok(None),
        }
    }
}
