//! Module that provides LLM client functionalities.
//! 
//! This module contains the implementation for interacting with Large Language Models (LLMs).

use reqwest::Client;
use serde::{Deserialize, Serialize};
use log::{error};
const SYSTEM_PROMPT: &str = "Ты — автоматический анализатор требований. Твоя задача — найти в переданном тексте проблемные фрагменты и вернуть строго структурированный набор записей в формате:
{quote: \"буквальная_цитата\",message: \"короткий_комментарий\",severity: \"error|warn|info\",category: \"Ambiguity|Contradiction|MissingDetail|Risk|Question|Dependency\"}
Записи нужно перечислить подряд через запятую без пробелов между запятыми (пример: {…},{…},{…}). Ничего больше — ни пояснений, ни заголовков, ни JSON-обёрток. Если находок нет — верни пустую строку.

Требования к содержимому записи:
- quote: буквальная выдержка из исходного текста, максимально короткая, но достаточная, чтобы увидеть проблему (вплоть до 200 символов). Обязательно брать текст дословно из входа.
- message: краткий комментарий (одно-два предложения), чётко и без разговоров: что не так и почему это проблема для реализации.
- severity: строго одно из трёх значений — error, warn, info. Никаких других слов.
- category: строго одна из: Ambiguity, Contradiction, MissingDetail, Risk, Question, Dependency.

Сопоставление категорий и уровней серьёзности (жёсткие правила):
- Contradiction → severity = error. (Противоречие в требованиях — блокер реализации.)
- MissingDetail → severity = error. (Отсутствие детали, которое делает реализацию невозможной/неопределённой.)
- Risk → severity = warn. (Упоминание риска, требующее внимания, но не обязательно блокер прямо сейчас.)
- Ambiguity → severity = warn. (Неоднозначная формулировка; требуется уточнение.)
- Dependency → severity = info. (Упоминание внешней зависимости/сервиса/модуля — полезно, но обычно не критично.)
- Question → severity = info. (Прямой вопрос/пункт для уточнения.)

Если конкретная находка по контексту очевидно должна иметь более высокую серьёзность (например, MissingDetail, который однозначно блокирует релиз), следуй таблице выше и выставляй error. Не придумывай промежуточных степеней.

Ограничения и поведение:
- Максимум выводимых записей: 15. Если находок больше — выбери самые релевантные по серьёзности и явности проблемы.
- Не дублируй близкие цитаты; агрегируй в одну запись, если проблема одна и та же.
- Не добавляй номера, метки или дополнительные поля.
- Всегда используйте заданные точные имена категорий и уровней серьёзности (чувствительность к регистру допускается, но лучше точно как указано).
- Результат должен соответствовать формату ровно: фигурные скобки, ключи в порядке quote, message, severity, category, двойные кавычки для значений. Пример корректного вывода:
  {quote:\"Пользователь должен быть аутентифицирован\",message:\"Не указано, каким способом и в каких случаях; нет требований по сессиям/токенам\",severity:\"error\",category:\"MissingDetail\"},{quote:\"Система должна работать быстро\",message:\"'быстро' не определено числово (SLA/латентность).\",severity:\"warn\",category:\"Ambiguity\"}

Входной текст придёт как одно сообщение от пользователя. Проанализируй весь текст и верни только требуемые записи в строгом формате.
";

#[derive(Deserialize)]
struct LLMResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: Message,
}

#[derive(Deserialize)]
struct Message {
    content: String,
}

pub enum LLMClientError {
    RequestError(reqwest::Error),
    StatusError(reqwest::StatusCode),
    ResponseParseError(reqwest::Error),
    SerdeError(serde_json::Error),
}

impl From<reqwest::Error> for LLMClientError {
    fn from(err: reqwest::Error) -> Self {
        LLMClientError::RequestError(err)
    }
}

impl From<serde_json::Error> for LLMClientError {
    fn from(err: serde_json::Error) -> Self {
        LLMClientError::SerdeError(err)
    }
}

impl std::fmt::Display for LLMClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LLMClientError::RequestError(e) => write!(f, "Request error: {}", e),
            LLMClientError::StatusError(code) => write!(f, "Non-OK status code: {}", code),
            LLMClientError::ResponseParseError(e) => write!(f, "Response parse error: {}", e),
            LLMClientError::SerdeError(e) => write!(f, "Serde error: {}", e),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LLMAnalysisIssue {
    pub quote: String,
    pub message: String, 
    pub severity: String,
    pub category: String,
}

pub struct LLMClient {
    client: Client,
    api_key: String,
    base_url: String,
    model: String,
    enable_reasoning: bool,
}

impl LLMClient {
    pub fn new(api_key: String, base_url: String, model: String, enable_reasoning: bool ) -> Self {
        LLMClient {
            client: Client::new(),
            api_key,
            base_url,
            model,
            enable_reasoning,
        }
    }

    pub async fn analyze_input(&self, input: &str) -> Result<Vec<LLMAnalysisIssue>, LLMClientError> {
        let body = if self.enable_reasoning {serde_json::json!({
            "model": self.model,
            "messages": [
                {
                    "role": "system",
                    "content": SYSTEM_PROMPT
                },
                {
                    "role": "user",
                    "content": input
                }
            ],
            
            "extra_body": {
                "reasoning": {"enabled": true}
            }
        })} else {
            serde_json::json!({
                "model": self.model,
                "messages": [
                    {
                        "role": "system",
                        "content": SYSTEM_PROMPT
                    },
                    {
                        "role": "user",
                        "content": input
                    }
                ]
            })
        };



        let response = self.client.post(&self.base_url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&body)
            .send()
            .await.map_err(|e| {
                error!("Error sending request to LLM: {}", e);
                e
            })?;

        if response.status() != reqwest::StatusCode::OK {
            let status = response.status();
            error!("LLM returned non-OK status: {}", status);
            let error_body = response.text().await.unwrap_or_else(|_| "Unable to read error body".to_string());
            error!("Error response body: {}", error_body);
            return Err(LLMClientError::StatusError(status));
        }

        let response_json: LLMResponse = response.json().await.map_err(|e| {
            error!("Error parsing LLM response JSON: {}", e);
            LLMClientError::ResponseParseError(e)
        })?;
        let issues: Vec<LLMAnalysisIssue> = self.parse_issues(&response_json.choices[0].message.content);
        Ok(issues)
    }

    pub fn parse_issues(&self, content: &str) -> Vec<LLMAnalysisIssue> {
        let mut issues = Vec::new();
        for item in content.split("},{") {
            let item = item.trim_matches(|c| c == '{' || c == '}');
            let mut quote = String::new();
            let mut message = String::new();
            let mut severity = String::new();
            let mut category = String::new();

            for pair in item.split(',') {
                let mut kv = pair.splitn(2, ':');
                let key = kv.next().unwrap_or("").trim_matches('"');
                let value = kv.next().unwrap_or("").trim_matches('"');

                match key {
                    "quote" => quote = value.to_string(),
                    "message" => message = value.to_string(),
                    "severity" => severity = value.to_string(),
                    "category" => category = value.to_string(),
                    _ => {}
                }
            }

            if !quote.is_empty() && !message.is_empty() && !severity.is_empty() && !category.is_empty() {
                issues.push(LLMAnalysisIssue {
                    quote,
                    message,
                    severity,
                    category,
                });
            }
        }
        issues
    }
}