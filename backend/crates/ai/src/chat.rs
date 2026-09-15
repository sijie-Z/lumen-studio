use common::AppError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest {
    pub messages: Vec<ChatMessage>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChatResponse {
    pub reply: String,
    pub mode: String,
    pub sources: Vec<String>,
}

#[derive(Clone)]
pub struct ChatClient {
    http: reqwest::Client,
    base_url: String,
    api_key: Option<String>,
    model: String,
}

impl ChatClient {
    pub fn from_env() -> Self {
        Self {
            http: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(45))
                .build()
                .expect("valid reqwest client"),
            base_url: std::env::var("OPENAI_BASE_URL")
                .unwrap_or_else(|_| "https://api.openai.com/v1".into()),
            api_key: std::env::var("OPENAI_API_KEY")
                .ok()
                .filter(|key| !key.trim().is_empty()),
            model: std::env::var("OPENAI_MODEL").unwrap_or_else(|_| "gpt-4.1-mini".into()),
        }
    }

    pub async fn chat(&self, request: &ChatRequest) -> Result<ChatResponse, AppError> {
        if self.api_key.is_some() {
            self.chat_remote(request).await
        } else {
            Ok(local_reply(request))
        }
    }

    async fn chat_remote(&self, request: &ChatRequest) -> Result<ChatResponse, AppError> {
        let endpoint = format!("{}/chat/completions", self.base_url.trim_end_matches('/'));
        let payload = serde_json::json!({
            "model": self.model,
            "messages": request.messages,
            "temperature": 0.7
        });

        let response = self
            .http
            .post(endpoint)
            .bearer_auth(self.api_key.as_deref().unwrap_or_default())
            .json(&payload)
            .send()
            .await
            .map_err(AppError::from_anyhow)?;

        let status = response.status();
        let body: serde_json::Value = response
            .json()
            .await
            .map_err(AppError::from_anyhow)?;
        if !status.is_success() {
            let message = body["error"]["message"]
                .as_str()
                .unwrap_or("upstream AI request failed");
            return Err(AppError::BadRequest(format!("AI service error: {message}")));
        }

        let reply = body["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("抱歉，我暂时没有生成回复。")
            .trim()
            .to_string();

        Ok(ChatResponse {
            reply,
            mode: "llm".into(),
            sources: vec![],
        })
    }
}

fn local_reply(request: &ChatRequest) -> ChatResponse {
    let message = request
        .messages
        .last()
        .map(|item| item.content.as_str())
        .unwrap_or("");

    let (reply, sources) = if message.contains("预约") || message.contains("拍摄") {
        (
            "预约流程：选择创作者与服务后提交预约，创作者确认即可锁定档期；支付成功后进入待拍摄状态。需要我帮你推荐创作者吗？".into(),
            vec!["预约流程".into(), "#/services".into()],
        )
    } else if message.contains("价格") || message.contains("收费") {
        (
            "价格由创作者按服务类型、时长与交付数量设定。你可以浏览服务详情获取报价，或告诉我预算和风格，我来匹配。".into(),
            vec!["服务报价".into(), "#/services".into()],
        )
    } else if message.contains("摄影师") || message.contains("创作者") {
        (
            "平台已收录人像、婚礼、商业与短片创作者。描述你想要的风格、地点和预算，我可以给你更精准的推荐。".into(),
            vec!["创作者发现".into(), "#/creators".into()],
        )
    } else if message.contains("风格") {
        (
            "我们的风格 DNA 系统会从色调、构图和情绪三个维度分析作品。说说你喜欢的画面感，比如暖色、纪实或安静。".into(),
            vec!["风格匹配".into(), "#/ai".into()],
        )
    } else {
        (
            "你好，我是 Lumina 创意助手。我可以帮你推荐创作者、查询服务与价格、梳理预约流程，或者根据你的描述匹配视觉风格。你想从哪里开始？".into(),
            vec!["AI 助手".into()],
        )
    };

    ChatResponse {
        reply,
        mode: "local".into(),
        sources,
    }
}
