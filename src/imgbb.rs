use anyhow::{Context, Result};
use reqwest::multipart::{Form, Part};
use reqwest::Client;
use serde::Deserialize;
use std::time::Duration;

// 匹配新图床的 JSON 结构 (根据 jq -r '.links.share' 推断)
#[derive(Deserialize, Debug)]
struct KvaultResponse {
    links: Option<KvaultLinks>,
    // 预留错误信息字段，视你图床实际返回格式而定
    message: Option<String>, 
}

#[derive(Deserialize, Debug)]
struct KvaultLinks {
    share: String, // 你也可以根据需要改成 download
}

#[derive(Clone)]
pub struct KvaultUploader {
    pub base_url: String,
    pub api_token: String,
    client: Client,
}

impl KvaultUploader {
    pub fn new(base_url: &str, api_token: &str) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            api_token: api_token.to_string(),
            client: Client::builder()
                .timeout(Duration::from_secs(60))
                .build()
                .unwrap(),
        }
    }

    pub async fn upload_file(&self, file_name: &str, file_bytes: &[u8]) -> Result<String> {
        // 根据 API 指南，字段名为 "file"，无需附带 key
        let form = Form::new()
            .part("file", Part::bytes(file_bytes.to_vec()).file_name(file_name.to_string()));

        let upload_url = format!("{}/api/v1/upload", self.base_url);

        let res = self.client
            .post(&upload_url)
            .header("Authorization", format!("Bearer {}", self.api_token)) // Bearer 鉴权
            .multipart(form)
            .header("User-Agent", "exloli-client/3.0")
            .send()
            .await?;

        let status = res.status();
        let text = res.text().await.context("无法读取图床响应体")?;

        if status.is_success() {
            let parsed: KvaultResponse = serde_json::from_str(&text)
                .context(format!("JSON 解析失败: {}", text))?;
            
            if let Some(links) = parsed.links {
                Ok(links.share)
            } else if let Some(msg) = parsed.message {
                Err(anyhow::anyhow!("API 拒绝请求: {}", msg))
            } else {
                Err(anyhow::anyhow!("未知的 JSON 格式或未返回链接: {}", text))
            }
        } else {
            Err(anyhow::anyhow!("上传失败，HTTP 状态码: {}, 内容: {}", status, text))
        }
    }
}
