use crate::db::models::{
    resolve_provider_api_format, PROVIDER_API_GEMINI_GENERATE_CONTENT,
    PROVIDER_API_OPENAI_CHAT, PROVIDER_API_OPENAI_RESPONSES,
};
use crate::services::proxy::{CliType, TokenUsage};
use serde_json::{json, Map, Value};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApiFormat {
    AnthropicMessages,
    OpenAiChatCompletions,
    OpenAiResponses,
    GeminiGenerateContent,
}

impl ApiFormat {
    pub fn from_provider(cli_type: &str, api_format: Option<&str>) -> Self {
        match resolve_provider_api_format(cli_type, api_format) {
            PROVIDER_API_OPENAI_CHAT => Self::OpenAiChatCompletions,
            PROVIDER_API_OPENAI_RESPONSES => Self::OpenAiResponses,
            PROVIDER_API_GEMINI_GENERATE_CONTENT => Self::GeminiGenerateContent,
            _ => Self::AnthropicMessages,
        }
    }

    pub fn from_client_request(cli_type: CliType, path: &str) -> Self {
        match cli_type {
            CliType::ClaudeCode => Self::AnthropicMessages,
            CliType::Codex => {
                if path.contains("/chat/completions") {
                    Self::OpenAiChatCompletions
                } else {
                    Self::OpenAiResponses
                }
            }
            CliType::Gemini => Self::GeminiGenerateContent,
        }
    }

    pub fn request_path(&self, fallback_path: &str) -> String {
        match self {
            Self::AnthropicMessages => "/v1/messages".to_string(),
            Self::OpenAiChatCompletions => "/v1/chat/completions".to_string(),
            Self::OpenAiResponses => "/v1/responses".to_string(),
            Self::GeminiGenerateContent => fallback_path.to_string(),
        }
    }

    pub fn content_type(&self, streaming: bool) -> &'static str {
        if streaming {
            "text/event-stream"
        } else {
            "application/json"
        }
    }
}

#[derive(Debug, Clone)]
pub struct TransformedRequest {
    pub path: String,
    pub body: Vec<u8>,
}

#[derive(Debug, Clone, Default)]
struct GenericRequest {
    model: Option<String>,
    stream: bool,
    max_output_tokens: Option<i64>,
    items: Vec<GenericItem>,
    tools: Vec<GenericToolDefinition>,
    tool_choice: Option<Value>,
}

#[derive(Debug, Clone)]
enum GenericItem {
    System(String),
    UserText(String),
    AssistantText(String),
    AssistantToolCall(GenericToolCall),
    ToolResult { call_id: String, output: String },
}

#[derive(Debug, Clone, Default)]
pub struct GenericToolCall {
    pub id: String,
    pub name: String,
    pub arguments: String,
}

#[derive(Debug, Clone, Default)]
struct GenericToolDefinition {
    name: String,
    description: Option<String>,
    parameters: Value,
}

#[derive(Debug, Clone, Default)]
pub struct GenericResponse {
    pub id: String,
    pub model: Option<String>,
    pub created_at: i64,
    pub text: String,
    pub tool_calls: Vec<GenericToolCall>,
    pub usage: TokenUsage,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Clone)]
enum ResponsesOutputItem {
    Text {
        output_index: usize,
        item_id: String,
        text: String,
    },
    FunctionCall {
        output_index: usize,
        item_id: String,
        call_id: String,
        name: String,
        arguments: String,
    },
}

impl ResponsesOutputItem {
    fn output_index(&self) -> usize {
        match self {
            Self::Text { output_index, .. } | Self::FunctionCall { output_index, .. } => {
                *output_index
            }
        }
    }

    fn added_item(&self) -> Value {
        match self {
            Self::Text { item_id, .. } => json!({
                "id": item_id,
                "type": "message",
                "role": "assistant",
                "status": "in_progress",
                "content": [{
                    "type": "output_text",
                    "text": ""
                }]
            }),
            Self::FunctionCall {
                item_id,
                call_id,
                name,
                ..
            } => json!({
                "id": item_id,
                "type": "function_call",
                "call_id": call_id,
                "name": name,
                "arguments": "",
                "status": "in_progress"
            }),
        }
    }

    fn completed_item(&self) -> Value {
        match self {
            Self::Text { item_id, text, .. } => json!({
                "id": item_id,
                "type": "message",
                "role": "assistant",
                "status": "completed",
                "content": [{
                    "type": "output_text",
                    "text": text
                }]
            }),
            Self::FunctionCall {
                item_id,
                call_id,
                name,
                arguments,
                ..
            } => json!({
                "id": item_id,
                "type": "function_call",
                "call_id": call_id,
                "name": name,
                "arguments": arguments,
                "status": "completed"
            }),
        }
    }
}

pub fn transform_request(
    source_api: ApiFormat,
    target_api: ApiFormat,
    path: &str,
    body: &[u8],
    default_chat_max_tokens: i64,
) -> Result<TransformedRequest, String> {
    if source_api == target_api {
        return Ok(TransformedRequest {
            path: target_api.request_path(path),
            body: serialize_passthrough_request(target_api, body, default_chat_max_tokens)?,
        });
    }

    if matches!(source_api, ApiFormat::GeminiGenerateContent)
        || matches!(target_api, ApiFormat::GeminiGenerateContent)
    {
        return Err("Gemini 供应商暂不支持跨协议转换".to_string());
    }

    let mut generic = parse_request(source_api, body)?;
    if matches!(target_api, ApiFormat::OpenAiChatCompletions) && generic.max_output_tokens.is_none()
    {
        generic.max_output_tokens = Some(default_chat_max_tokens);
    }
    let body = serde_json::to_vec(&serialize_request(&generic, target_api))
        .map_err(|e| format!("序列化转换后的请求失败: {}", e))?;

    Ok(TransformedRequest {
        path: target_api.request_path(path),
        body,
    })
}

pub fn transform_response_body(
    target_api: ApiFormat,
    source_api: ApiFormat,
    body: &[u8],
) -> Result<Vec<u8>, String> {
    if source_api == target_api {
        return Ok(body.to_vec());
    }

    if matches!(source_api, ApiFormat::GeminiGenerateContent)
        || matches!(target_api, ApiFormat::GeminiGenerateContent)
    {
        return Err("Gemini 供应商暂不支持跨协议转换".to_string());
    }

    let generic = parse_response_body(target_api, body)?;
    serde_json::to_vec(&serialize_response_body(source_api, &generic))
        .map_err(|e| format!("序列化转换后的响应失败: {}", e))
}

pub fn transform_streaming_response(
    target_api: ApiFormat,
    source_api: ApiFormat,
    body: &[u8],
) -> Result<Vec<u8>, String> {
    if source_api == target_api {
        return Ok(body.to_vec());
    }

    if matches!(source_api, ApiFormat::GeminiGenerateContent)
        || matches!(target_api, ApiFormat::GeminiGenerateContent)
    {
        return Err("Gemini 供应商暂不支持跨协议转换".to_string());
    }

    let generic = parse_streaming_response(target_api, body)?;
    Ok(serialize_streaming_response(source_api, &generic))
}

pub fn parse_token_usage_by_api(data: &[u8], api: ApiFormat, usage: &mut TokenUsage) {
    let Ok(json) = serde_json::from_slice::<Value>(data) else {
        return;
    };

    match api {
        ApiFormat::AnthropicMessages => {
            if let Some(msg_usage) = json.get("message").and_then(|m| m.get("usage")) {
                if let Some(input) = msg_usage.get("input_tokens").and_then(|v| v.as_i64()) {
                    usage.input_tokens = input;
                }
                if let Some(output) = msg_usage.get("output_tokens").and_then(|v| v.as_i64()) {
                    usage.output_tokens = output;
                }
            } else if let Some(root_usage) = json.get("usage") {
                if let Some(input) = root_usage.get("input_tokens").and_then(|v| v.as_i64()) {
                    usage.input_tokens = input;
                }
                if let Some(output) = root_usage.get("output_tokens").and_then(|v| v.as_i64()) {
                    usage.output_tokens = output;
                }
            }
        }
        ApiFormat::OpenAiChatCompletions => {
            if let Some(root_usage) = json.get("usage") {
                if let Some(input) = root_usage
                    .get("prompt_tokens")
                    .or_else(|| root_usage.get("input_tokens"))
                    .and_then(|v| v.as_i64())
                {
                    usage.input_tokens = input;
                }
                if let Some(output) = root_usage
                    .get("completion_tokens")
                    .or_else(|| root_usage.get("output_tokens"))
                    .and_then(|v| v.as_i64())
                {
                    usage.output_tokens = output;
                }
            }
        }
        ApiFormat::OpenAiResponses => {
            let usage_node = json
                .get("response")
                .and_then(|response| response.get("usage"))
                .or_else(|| json.get("usage"));
            if let Some(usage_node) = usage_node {
                if let Some(input) = usage_node.get("input_tokens").and_then(|v| v.as_i64()) {
                    usage.input_tokens = input;
                }
                if let Some(output) = usage_node.get("output_tokens").and_then(|v| v.as_i64()) {
                    usage.output_tokens = output;
                }
            }
        }
        ApiFormat::GeminiGenerateContent => {
            if let Some(metadata) = json.get("usageMetadata") {
                if let Some(prompt) = metadata.get("promptTokenCount").and_then(|v| v.as_i64()) {
                    usage.input_tokens = prompt;
                }
                let candidates = metadata
                    .get("candidatesTokenCount")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0);
                let thoughts = metadata
                    .get("thoughtsTokenCount")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0);
                usage.output_tokens = candidates + thoughts;
            }
        }
    }
}

fn parse_request(api: ApiFormat, body: &[u8]) -> Result<GenericRequest, String> {
    let json =
        serde_json::from_slice::<Value>(body).map_err(|e| format!("解析请求体失败: {}", e))?;
    match api {
        ApiFormat::AnthropicMessages => parse_anthropic_request(&json),
        ApiFormat::OpenAiChatCompletions => parse_chat_request(&json),
        ApiFormat::OpenAiResponses => parse_responses_request(&json),
        ApiFormat::GeminiGenerateContent => Err("Gemini 请求暂不支持转换".to_string()),
    }
}

fn serialize_request(request: &GenericRequest, api: ApiFormat) -> Value {
    match api {
        ApiFormat::AnthropicMessages => serialize_anthropic_request(request),
        ApiFormat::OpenAiChatCompletions => serialize_chat_request(request),
        ApiFormat::OpenAiResponses => serialize_responses_request(request),
        ApiFormat::GeminiGenerateContent => Value::Null,
    }
}

fn serialize_passthrough_request(
    api: ApiFormat,
    body: &[u8],
    default_chat_max_tokens: i64,
) -> Result<Vec<u8>, String> {
    match api {
        ApiFormat::OpenAiChatCompletions => {
            inject_chat_request_max_tokens(body, default_chat_max_tokens)
        }
        _ => Ok(body.to_vec()),
    }
}

fn inject_chat_request_max_tokens(
    body: &[u8],
    default_chat_max_tokens: i64,
) -> Result<Vec<u8>, String> {
    let mut json =
        serde_json::from_slice::<Value>(body).map_err(|e| format!("解析请求体失败: {}", e))?;
    let Some(root) = json.as_object_mut() else {
        return Ok(body.to_vec());
    };
    if !root.contains_key("max_tokens") {
        let max_tokens = root
            .get("max_completion_tokens")
            .and_then(|value| value.as_i64())
            .unwrap_or(default_chat_max_tokens);
        root.insert("max_tokens".to_string(), Value::Number(max_tokens.into()));
    }
    serde_json::to_vec(&json).map_err(|e| format!("序列化转换后的请求失败: {}", e))
}

fn parse_response_body(api: ApiFormat, body: &[u8]) -> Result<GenericResponse, String> {
    let json =
        serde_json::from_slice::<Value>(body).map_err(|e| format!("解析响应体失败: {}", e))?;
    match api {
        ApiFormat::AnthropicMessages => Ok(parse_anthropic_response(&json)),
        ApiFormat::OpenAiChatCompletions => Ok(parse_chat_response(&json)),
        ApiFormat::OpenAiResponses => Ok(parse_responses_response(&json)),
        ApiFormat::GeminiGenerateContent => Err("Gemini 响应暂不支持转换".to_string()),
    }
}

fn serialize_response_body(api: ApiFormat, response: &GenericResponse) -> Value {
    match api {
        ApiFormat::AnthropicMessages => serialize_anthropic_response(response),
        ApiFormat::OpenAiChatCompletions => serialize_chat_response(response),
        ApiFormat::OpenAiResponses => serialize_responses_response(response),
        ApiFormat::GeminiGenerateContent => Value::Null,
    }
}

fn parse_anthropic_request(json: &Value) -> Result<GenericRequest, String> {
    let mut request = GenericRequest {
        model: json
            .get("model")
            .and_then(|v| v.as_str())
            .map(|v| v.to_string()),
        stream: json
            .get("stream")
            .and_then(|v| v.as_bool())
            .unwrap_or(false),
        max_output_tokens: json.get("max_tokens").and_then(|v| v.as_i64()),
        tool_choice: json.get("tool_choice").cloned(),
        ..Default::default()
    };

    if let Some(system_text) = extract_anthropic_system_text(json.get("system")) {
        request.items.push(GenericItem::System(system_text));
    }

    if let Some(messages) = json.get("messages").and_then(|v| v.as_array()) {
        for message in messages {
            let role = message.get("role").and_then(|v| v.as_str()).unwrap_or("");
            for block in anthropic_blocks(message.get("content")) {
                match (role, block.get("type").and_then(|v| v.as_str())) {
                    ("user", Some("text")) => {
                        if let Some(text) = block.get("text").and_then(|v| v.as_str()) {
                            request.items.push(GenericItem::UserText(text.to_string()));
                        }
                    }
                    ("assistant", Some("text")) => {
                        if let Some(text) = block.get("text").and_then(|v| v.as_str()) {
                            request
                                .items
                                .push(GenericItem::AssistantText(text.to_string()));
                        }
                    }
                    ("assistant", Some("tool_use")) => {
                        request
                            .items
                            .push(GenericItem::AssistantToolCall(GenericToolCall {
                                id: block
                                    .get("id")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or_default()
                                    .to_string(),
                                name: block
                                    .get("name")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or_default()
                                    .to_string(),
                                arguments: block
                                    .get("input")
                                    .map(value_to_argument_string)
                                    .unwrap_or_else(|| "{}".to_string()),
                            }))
                    }
                    ("user", Some("tool_result")) => request.items.push(GenericItem::ToolResult {
                        call_id: block
                            .get("tool_use_id")
                            .and_then(|v| v.as_str())
                            .unwrap_or_default()
                            .to_string(),
                        output: block.get("content").map(extract_text).unwrap_or_default(),
                    }),
                    _ => {}
                }
            }
        }
    }

    if let Some(tools) = json.get("tools").and_then(|v| v.as_array()) {
        request.tools = tools
            .iter()
            .map(|tool| GenericToolDefinition {
                name: tool
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string(),
                description: tool
                    .get("description")
                    .and_then(|v| v.as_str())
                    .map(|v| v.to_string()),
                parameters: tool
                    .get("input_schema")
                    .cloned()
                    .unwrap_or_else(|| json!({})),
            })
            .collect();
    }

    Ok(request)
}

fn parse_chat_request(json: &Value) -> Result<GenericRequest, String> {
    let mut request = GenericRequest {
        model: json
            .get("model")
            .and_then(|v| v.as_str())
            .map(|v| v.to_string()),
        stream: json
            .get("stream")
            .and_then(|v| v.as_bool())
            .unwrap_or(false),
        max_output_tokens: json
            .get("max_tokens")
            .or_else(|| json.get("max_completion_tokens"))
            .and_then(|v| v.as_i64()),
        tool_choice: json.get("tool_choice").cloned(),
        ..Default::default()
    };

    if let Some(messages) = json.get("messages").and_then(|v| v.as_array()) {
        for message in messages {
            match message.get("role").and_then(|v| v.as_str()) {
                Some("system") => {
                    let text = extract_text(message.get("content").unwrap_or(&Value::Null));
                    if !text.is_empty() {
                        request.items.push(GenericItem::System(text));
                    }
                }
                Some("user") => {
                    let text = extract_text(message.get("content").unwrap_or(&Value::Null));
                    if !text.is_empty() {
                        request.items.push(GenericItem::UserText(text));
                    }
                }
                Some("assistant") => {
                    let text = extract_text(message.get("content").unwrap_or(&Value::Null));
                    if !text.is_empty() {
                        request.items.push(GenericItem::AssistantText(text));
                    }
                    if let Some(tool_calls) = message.get("tool_calls").and_then(|v| v.as_array()) {
                        for tool in tool_calls {
                            request
                                .items
                                .push(GenericItem::AssistantToolCall(GenericToolCall {
                                    id: tool
                                        .get("id")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or_default()
                                        .to_string(),
                                    name: tool
                                        .get("function")
                                        .and_then(|v| v.get("name"))
                                        .and_then(|v| v.as_str())
                                        .unwrap_or_default()
                                        .to_string(),
                                    arguments: tool
                                        .get("function")
                                        .and_then(|v| v.get("arguments"))
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("{}")
                                        .to_string(),
                                }));
                        }
                    }
                }
                Some("tool") => request.items.push(GenericItem::ToolResult {
                    call_id: message
                        .get("tool_call_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or_default()
                        .to_string(),
                    output: extract_text(message.get("content").unwrap_or(&Value::Null)),
                }),
                _ => {}
            }
        }
    }

    if let Some(tools) = json.get("tools").and_then(|v| v.as_array()) {
        request.tools = tools
            .iter()
            .map(|tool| {
                let function = tool.get("function").unwrap_or(tool);
                GenericToolDefinition {
                    name: function
                        .get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or_default()
                        .to_string(),
                    description: function
                        .get("description")
                        .and_then(|v| v.as_str())
                        .map(|v| v.to_string()),
                    parameters: function
                        .get("parameters")
                        .cloned()
                        .unwrap_or_else(|| json!({})),
                }
            })
            .collect();
    }

    Ok(request)
}

fn parse_responses_request(json: &Value) -> Result<GenericRequest, String> {
    let mut request = GenericRequest {
        model: json
            .get("model")
            .and_then(|v| v.as_str())
            .map(|v| v.to_string()),
        stream: json
            .get("stream")
            .and_then(|v| v.as_bool())
            .unwrap_or(false),
        max_output_tokens: json.get("max_output_tokens").and_then(|v| v.as_i64()),
        tool_choice: json.get("tool_choice").cloned(),
        ..Default::default()
    };

    if let Some(instructions) = json.get("instructions").and_then(|v| v.as_str()) {
        if !instructions.is_empty() {
            request
                .items
                .push(GenericItem::System(instructions.to_string()));
        }
    }

    match json.get("input") {
        Some(Value::String(text)) if !text.is_empty() => {
            request.items.push(GenericItem::UserText(text.to_string()))
        }
        Some(Value::Array(items)) => {
            for item in items {
                if let Some(role) = item.get("role").and_then(|v| v.as_str()) {
                    let text = extract_text(item.get("content").unwrap_or(&Value::Null));
                    match role {
                        "system" if !text.is_empty() => {
                            request.items.push(GenericItem::System(text))
                        }
                        "user" if !text.is_empty() => {
                            request.items.push(GenericItem::UserText(text))
                        }
                        "assistant" if !text.is_empty() => {
                            request.items.push(GenericItem::AssistantText(text))
                        }
                        _ => {}
                    }
                    continue;
                }

                match item.get("type").and_then(|v| v.as_str()) {
                    Some("function_call") => {
                        request
                            .items
                            .push(GenericItem::AssistantToolCall(GenericToolCall {
                                id: item
                                    .get("call_id")
                                    .or_else(|| item.get("id"))
                                    .and_then(|v| v.as_str())
                                    .unwrap_or_default()
                                    .to_string(),
                                name: item
                                    .get("name")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or_default()
                                    .to_string(),
                                arguments: item
                                    .get("arguments")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("{}")
                                    .to_string(),
                            }))
                    }
                    Some("function_call_output") => request.items.push(GenericItem::ToolResult {
                        call_id: item
                            .get("call_id")
                            .and_then(|v| v.as_str())
                            .unwrap_or_default()
                            .to_string(),
                        output: extract_text(item.get("output").unwrap_or(&Value::Null)),
                    }),
                    _ => {}
                }
            }
        }
        _ => {}
    }

    if let Some(tools) = json.get("tools").and_then(|v| v.as_array()) {
        request.tools = tools
            .iter()
            .filter_map(parse_responses_tool_definition)
            .collect();
    }

    Ok(request)
}

fn serialize_anthropic_request(request: &GenericRequest) -> Value {
    let mut root = Map::new();
    if let Some(model) = &request.model {
        root.insert("model".to_string(), Value::String(model.clone()));
    }
    root.insert("stream".to_string(), Value::Bool(request.stream));
    if let Some(max_tokens) = request.max_output_tokens {
        root.insert("max_tokens".to_string(), Value::Number(max_tokens.into()));
    }
    if !request.tools.is_empty() {
        root.insert(
            "tools".to_string(),
            Value::Array(
                request
                    .tools
                    .iter()
                    .map(|tool| {
                        json!({
                            "name": tool.name,
                            "description": tool.description,
                            "input_schema": tool.parameters
                        })
                    })
                    .collect(),
            ),
        );
    }
    if let Some(tool_choice) = request
        .tool_choice
        .as_ref()
        .and_then(convert_tool_choice_to_anthropic)
    {
        root.insert("tool_choice".to_string(), tool_choice);
    }

    let system_items: Vec<Value> = request
        .items
        .iter()
        .filter_map(|item| match item {
            GenericItem::System(text) => Some(json!({"type": "text", "text": text})),
            _ => None,
        })
        .collect();
    if !system_items.is_empty() {
        root.insert("system".to_string(), Value::Array(system_items));
    }

    let mut messages = Vec::<Value>::new();
    for item in &request.items {
        match item {
            GenericItem::System(_) => {}
            GenericItem::UserText(text) => {
                push_anthropic_block(&mut messages, "user", json!({"type": "text", "text": text}))
            }
            GenericItem::AssistantText(text) => push_anthropic_block(
                &mut messages,
                "assistant",
                json!({"type": "text", "text": text}),
            ),
            GenericItem::AssistantToolCall(tool_call) => push_anthropic_block(
                &mut messages,
                "assistant",
                json!({
                    "type": "tool_use",
                    "id": non_empty_or_generated(&tool_call.id, "toolu"),
                    "name": tool_call.name,
                    "input": parse_arguments_json(&tool_call.arguments)
                }),
            ),
            GenericItem::ToolResult { call_id, output } => push_anthropic_block(
                &mut messages,
                "user",
                json!({
                    "type": "tool_result",
                    "tool_use_id": call_id,
                    "content": output
                }),
            ),
        }
    }
    root.insert("messages".to_string(), Value::Array(messages));
    Value::Object(root)
}

fn serialize_chat_request(request: &GenericRequest) -> Value {
    let mut root = Map::new();
    if let Some(model) = &request.model {
        root.insert("model".to_string(), Value::String(model.clone()));
    }
    root.insert("stream".to_string(), Value::Bool(request.stream));
    if let Some(max_tokens) = request.max_output_tokens {
        root.insert("max_tokens".to_string(), Value::Number(max_tokens.into()));
    }
    let chat_tools: Vec<&GenericToolDefinition> = request
        .tools
        .iter()
        .filter(|tool| !tool.name.trim().is_empty())
        .collect();
    if !chat_tools.is_empty() {
        root.insert(
            "tools".to_string(),
            Value::Array(
                chat_tools
                    .iter()
                    .map(|tool| {
                        json!({
                            "type": "function",
                            "function": {
                                "name": tool.name,
                                "description": tool.description,
                                "parameters": tool.parameters
                            }
                        })
                    })
                    .collect(),
            ),
        );
    }
    if !chat_tools.is_empty() {
        if let Some(tool_choice) = request
            .tool_choice
            .as_ref()
            .and_then(convert_tool_choice_to_chat)
        {
            root.insert("tool_choice".to_string(), tool_choice);
        }
    }

    let mut messages = Vec::<Value>::new();
    let mut current_assistant: Option<(String, Vec<Value>)> = None;
    for item in &request.items {
        match item {
            GenericItem::System(text) => {
                flush_assistant_message(&mut messages, &mut current_assistant);
                messages.push(json!({"role": "system", "content": text}));
            }
            GenericItem::UserText(text) => {
                flush_assistant_message(&mut messages, &mut current_assistant);
                messages.push(json!({"role": "user", "content": text}));
            }
            GenericItem::ToolResult { call_id, output } => {
                flush_assistant_message(&mut messages, &mut current_assistant);
                messages.push(json!({
                    "role": "tool",
                    "tool_call_id": call_id,
                    "content": output
                }));
            }
            GenericItem::AssistantText(text) => {
                let current = current_assistant.get_or_insert_with(|| (String::new(), Vec::new()));
                current.0.push_str(text);
            }
            GenericItem::AssistantToolCall(tool_call) => {
                let current = current_assistant.get_or_insert_with(|| (String::new(), Vec::new()));
                current.1.push(json!({
                    "id": non_empty_or_generated(&tool_call.id, "call"),
                    "type": "function",
                    "function": {
                        "name": tool_call.name,
                        "arguments": tool_call.arguments
                    }
                }));
            }
        }
    }
    flush_assistant_message(&mut messages, &mut current_assistant);

    root.insert("messages".to_string(), Value::Array(messages));
    Value::Object(root)
}

fn serialize_responses_request(request: &GenericRequest) -> Value {
    let mut root = Map::new();
    if let Some(model) = &request.model {
        root.insert("model".to_string(), Value::String(model.clone()));
    }
    root.insert("stream".to_string(), Value::Bool(request.stream));
    if let Some(max_tokens) = request.max_output_tokens {
        root.insert(
            "max_output_tokens".to_string(),
            Value::Number(max_tokens.into()),
        );
    }
    if !request.tools.is_empty() {
        root.insert(
            "tools".to_string(),
            Value::Array(
                request
                    .tools
                    .iter()
                    .map(|tool| {
                        json!({
                            "type": "function",
                            "name": tool.name,
                            "description": tool.description,
                            "parameters": tool.parameters
                        })
                    })
                    .collect(),
            ),
        );
    }
    if let Some(tool_choice) = request
        .tool_choice
        .as_ref()
        .and_then(convert_tool_choice_to_responses)
    {
        root.insert("tool_choice".to_string(), tool_choice);
    }

    let mut input = Vec::<Value>::new();
    for item in &request.items {
        match item {
            GenericItem::System(text) => input.push(json!({"role": "system", "content": text})),
            GenericItem::UserText(text) => input.push(json!({"role": "user", "content": text})),
            GenericItem::AssistantText(text) => {
                input.push(json!({"role": "assistant", "content": text}))
            }
            GenericItem::AssistantToolCall(tool_call) => input.push(json!({
                "type": "function_call",
                "call_id": non_empty_or_generated(&tool_call.id, "call"),
                "name": tool_call.name,
                "arguments": tool_call.arguments
            })),
            GenericItem::ToolResult { call_id, output } => input.push(json!({
                "type": "function_call_output",
                "call_id": call_id,
                "output": output
            })),
        }
    }
    root.insert("input".to_string(), Value::Array(input));
    Value::Object(root)
}

fn parse_anthropic_response(json: &Value) -> GenericResponse {
    let mut response = GenericResponse {
        id: json
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string(),
        model: json
            .get("model")
            .and_then(|v| v.as_str())
            .map(|v| v.to_string()),
        created_at: chrono::Utc::now().timestamp(),
        finish_reason: json
            .get("stop_reason")
            .and_then(|v| v.as_str())
            .map(|v| v.to_string()),
        ..Default::default()
    };
    if let Some(usage) = json.get("usage") {
        if let Some(input) = usage.get("input_tokens").and_then(|v| v.as_i64()) {
            response.usage.input_tokens = input;
        }
        if let Some(output) = usage.get("output_tokens").and_then(|v| v.as_i64()) {
            response.usage.output_tokens = output;
        }
    }
    if let Some(content) = json.get("content").and_then(|v| v.as_array()) {
        for item in content {
            match item.get("type").and_then(|v| v.as_str()) {
                Some("text") => {
                    if let Some(text) = item.get("text").and_then(|v| v.as_str()) {
                        response.text.push_str(text);
                    }
                }
                Some("tool_use") => response.tool_calls.push(GenericToolCall {
                    id: item
                        .get("id")
                        .and_then(|v| v.as_str())
                        .unwrap_or_default()
                        .to_string(),
                    name: item
                        .get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or_default()
                        .to_string(),
                    arguments: item
                        .get("input")
                        .map(value_to_argument_string)
                        .unwrap_or_else(|| "{}".to_string()),
                }),
                _ => {}
            }
        }
    }
    response
}

fn parse_chat_response(json: &Value) -> GenericResponse {
    let mut response = GenericResponse {
        id: json
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string(),
        model: json
            .get("model")
            .and_then(|v| v.as_str())
            .map(|v| v.to_string()),
        created_at: json
            .get("created")
            .and_then(|v| v.as_i64())
            .unwrap_or_else(|| chrono::Utc::now().timestamp()),
        ..Default::default()
    };

    if let Some(usage) = json.get("usage") {
        if let Some(input) = usage
            .get("prompt_tokens")
            .or_else(|| usage.get("input_tokens"))
            .and_then(|v| v.as_i64())
        {
            response.usage.input_tokens = input;
        }
        if let Some(output) = usage
            .get("completion_tokens")
            .or_else(|| usage.get("output_tokens"))
            .and_then(|v| v.as_i64())
        {
            response.usage.output_tokens = output;
        }
    }

    if let Some(choice) = json
        .get("choices")
        .and_then(|v| v.as_array())
        .and_then(|v| v.first())
    {
        response.finish_reason = choice
            .get("finish_reason")
            .and_then(|v| v.as_str())
            .map(|v| v.to_string());
        let message = choice.get("message").unwrap_or(&Value::Null);
        response.text = extract_chat_text(message);
        if let Some(tool_calls) = message.get("tool_calls").and_then(|v| v.as_array()) {
            response.tool_calls = tool_calls
                .iter()
                .map(|tool| GenericToolCall {
                    id: tool
                        .get("id")
                        .and_then(|v| v.as_str())
                        .unwrap_or_default()
                        .to_string(),
                    name: tool
                        .get("function")
                        .and_then(|v| v.get("name"))
                        .and_then(|v| v.as_str())
                        .unwrap_or_default()
                        .to_string(),
                    arguments: tool
                        .get("function")
                        .and_then(|v| v.get("arguments"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("{}")
                        .to_string(),
                })
                .collect();
        }
    }
    response
}

fn parse_responses_response(json: &Value) -> GenericResponse {
    let mut response = GenericResponse {
        id: json
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string(),
        model: json
            .get("model")
            .and_then(|v| v.as_str())
            .map(|v| v.to_string()),
        created_at: json
            .get("created_at")
            .and_then(|v| v.as_i64())
            .unwrap_or_else(|| chrono::Utc::now().timestamp()),
        finish_reason: json
            .get("status")
            .and_then(|v| v.as_str())
            .map(|v| v.to_string()),
        ..Default::default()
    };
    if let Some(usage) = json.get("usage") {
        if let Some(input) = usage.get("input_tokens").and_then(|v| v.as_i64()) {
            response.usage.input_tokens = input;
        }
        if let Some(output) = usage.get("output_tokens").and_then(|v| v.as_i64()) {
            response.usage.output_tokens = output;
        }
    }
    if let Some(output_items) = json.get("output").and_then(|v| v.as_array()) {
        for item in output_items {
            merge_response_output_item(&mut response, item);
        }
    }
    response
}

fn serialize_anthropic_response(response: &GenericResponse) -> Value {
    let mut content = Vec::<Value>::new();
    if !response.text.is_empty() {
        content.push(json!({"type": "text", "text": response.text}));
    }
    for tool_call in &response.tool_calls {
        content.push(json!({
            "type": "tool_use",
            "id": non_empty_or_generated(&tool_call.id, "toolu"),
            "name": tool_call.name,
            "input": parse_arguments_json(&tool_call.arguments)
        }));
    }

    json!({
        "id": non_empty_or_generated(&response.id, "msg"),
        "type": "message",
        "role": "assistant",
        "model": response.model.clone().unwrap_or_else(|| "ccg-gateway".to_string()),
        "content": content,
        "stop_reason": anthropic_finish_reason(response),
        "stop_sequence": null,
        "usage": {
            "input_tokens": response.usage.input_tokens,
            "output_tokens": response.usage.output_tokens
        }
    })
}

fn serialize_chat_response(response: &GenericResponse) -> Value {
    let content = if response.text.is_empty() {
        Value::Null
    } else {
        Value::String(response.text.clone())
    };
    let mut message = json!({
        "role": "assistant",
        "content": content
    });
    if !response.tool_calls.is_empty() {
        message["tool_calls"] = Value::Array(
            response
                .tool_calls
                .iter()
                .map(|tool_call| {
                    json!({
                        "id": non_empty_or_generated(&tool_call.id, "call"),
                        "type": "function",
                        "function": {
                            "name": tool_call.name,
                            "arguments": tool_call.arguments
                        }
                    })
                })
                .collect(),
        );
    }

    json!({
        "id": non_empty_or_generated(&response.id, "chatcmpl"),
        "object": "chat.completion",
        "created": response.created_at,
        "model": response.model.clone().unwrap_or_else(|| "ccg-gateway".to_string()),
        "choices": [{
            "index": 0,
            "message": message,
            "finish_reason": chat_finish_reason(response)
        }],
        "usage": {
            "prompt_tokens": response.usage.input_tokens,
            "completion_tokens": response.usage.output_tokens,
            "total_tokens": response.usage.input_tokens + response.usage.output_tokens
        }
    })
}

fn serialize_responses_response(response: &GenericResponse) -> Value {
    let response_id = non_empty_or_generated(&response.id, "resp");
    let model = response
        .model
        .clone()
        .unwrap_or_else(|| "ccg-gateway".to_string());
    let output_items = build_responses_output_items(response);

    serialize_responses_response_payload(response, &response_id, &model, &output_items)
}

fn parse_streaming_response(api: ApiFormat, body: &[u8]) -> Result<GenericResponse, String> {
    let frames = parse_sse_frames(body);
    if frames.is_empty() {
        let json = serde_json::from_slice::<Value>(body)
            .map_err(|e| format!("解析流式响应体失败: {}", e))?;
        if is_error_response_payload(&json) {
            return Err(extract_error_message(&json));
        }
        return match api {
            ApiFormat::AnthropicMessages => Ok(parse_anthropic_response(&json)),
            ApiFormat::OpenAiChatCompletions => Ok(parse_chat_response(&json)),
            ApiFormat::OpenAiResponses => Ok(parse_responses_response(&json)),
            ApiFormat::GeminiGenerateContent => Err("Gemini 响应暂不支持转换".to_string()),
        };
    }

    let mut response = GenericResponse {
        created_at: chrono::Utc::now().timestamp(),
        ..Default::default()
    };

    match api {
        ApiFormat::AnthropicMessages => {
            let mut active_tool_call: Option<usize> = None;
            for data in frames {
                let Ok(json) = serde_json::from_str::<Value>(&data) else {
                    continue;
                };
                match json.get("type").and_then(|v| v.as_str()) {
                    Some("message_start") => {
                        if let Some(message) = json.get("message") {
                            response.id = message
                                .get("id")
                                .and_then(|v| v.as_str())
                                .unwrap_or_default()
                                .to_string();
                            response.model = message
                                .get("model")
                                .and_then(|v| v.as_str())
                                .map(|v| v.to_string());
                        }
                    }
                    Some("content_block_start") => {
                        let block = json.get("content_block").unwrap_or(&Value::Null);
                        if block.get("type").and_then(|v| v.as_str()) == Some("tool_use") {
                            response.tool_calls.push(GenericToolCall {
                                id: block
                                    .get("id")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or_default()
                                    .to_string(),
                                name: block
                                    .get("name")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or_default()
                                    .to_string(),
                                arguments: String::new(),
                            });
                            active_tool_call = Some(response.tool_calls.len() - 1);
                        }
                    }
                    Some("content_block_delta") => {
                        let delta = json.get("delta").unwrap_or(&Value::Null);
                        match delta.get("type").and_then(|v| v.as_str()) {
                            Some("text_delta") => {
                                if let Some(text) = delta.get("text").and_then(|v| v.as_str()) {
                                    response.text.push_str(text);
                                }
                            }
                            Some("input_json_delta") => {
                                if let Some(index) = active_tool_call {
                                    if let Some(partial) =
                                        delta.get("partial_json").and_then(|v| v.as_str())
                                    {
                                        response.tool_calls[index].arguments.push_str(partial);
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                    Some("message_delta") => {
                        response.finish_reason = json
                            .get("delta")
                            .and_then(|v| v.get("stop_reason"))
                            .and_then(|v| v.as_str())
                            .map(|v| v.to_string());
                        if let Some(usage) = json.get("usage") {
                            if let Some(output) =
                                usage.get("output_tokens").and_then(|v| v.as_i64())
                            {
                                response.usage.output_tokens = output;
                            }
                        }
                    }
                    Some("message_stop") => active_tool_call = None,
                    _ => {}
                }
            }
        }
        ApiFormat::OpenAiChatCompletions => {
            for data in frames {
                if data == "[DONE]" {
                    continue;
                }
                let Ok(json) = serde_json::from_str::<Value>(&data) else {
                    continue;
                };
                if response.id.is_empty() {
                    response.id = json
                        .get("id")
                        .and_then(|v| v.as_str())
                        .unwrap_or_default()
                        .to_string();
                }
                if response.model.is_none() {
                    response.model = json
                        .get("model")
                        .and_then(|v| v.as_str())
                        .map(|v| v.to_string());
                }
                if let Some(choice) = json
                    .get("choices")
                    .and_then(|v| v.as_array())
                    .and_then(|v| v.first())
                {
                    if let Some(delta) = choice.get("delta") {
                        let text = extract_chat_text(delta);
                        if !text.is_empty() {
                            response.text.push_str(&text);
                        }
                        if let Some(tool_calls) = delta.get("tool_calls").and_then(|v| v.as_array())
                        {
                            for tool in tool_calls {
                                let index = tool.get("index").and_then(|v| v.as_u64()).unwrap_or(0)
                                    as usize;
                                while response.tool_calls.len() <= index {
                                    response.tool_calls.push(GenericToolCall::default());
                                }
                                if let Some(id) = tool.get("id").and_then(|v| v.as_str()) {
                                    response.tool_calls[index].id = id.to_string();
                                }
                                if let Some(name) = tool
                                    .get("function")
                                    .and_then(|v| v.get("name"))
                                    .and_then(|v| v.as_str())
                                {
                                    response.tool_calls[index].name = name.to_string();
                                }
                                if let Some(arguments) = tool
                                    .get("function")
                                    .and_then(|v| v.get("arguments"))
                                    .and_then(|v| v.as_str())
                                {
                                    response.tool_calls[index].arguments.push_str(arguments);
                                }
                            }
                        }
                    }
                    if let Some(reason) = choice.get("finish_reason").and_then(|v| v.as_str()) {
                        response.finish_reason = Some(reason.to_string());
                    }
                }
                parse_token_usage_by_api(&json.to_string().into_bytes(), api, &mut response.usage);
            }
        }
        ApiFormat::OpenAiResponses => {
            let mut output_index_to_tool_call = std::collections::HashMap::<usize, usize>::new();
            let mut item_id_to_tool_call = std::collections::HashMap::<String, usize>::new();

            for data in frames {
                let Ok(json) = serde_json::from_str::<Value>(&data) else {
                    continue;
                };
                match json.get("type").and_then(|v| v.as_str()) {
                    Some("response.created") => {
                        if let Some(inner) = json.get("response") {
                            response.id = inner
                                .get("id")
                                .and_then(|v| v.as_str())
                                .unwrap_or_default()
                                .to_string();
                            response.model = inner
                                .get("model")
                                .and_then(|v| v.as_str())
                                .map(|v| v.to_string());
                        }
                    }
                    Some("response.output_text.delta") => {
                        if let Some(delta) = json.get("delta").and_then(|v| v.as_str()) {
                            response.text.push_str(delta);
                        }
                    }
                    Some("response.output_text.done") => {
                        if let Some(text) = json.get("text").and_then(|v| v.as_str()) {
                            merge_response_text_done(&mut response, text);
                        }
                    }
                    Some("response.output_item.added") | Some("response.output_item.done") => {
                        if let Some(item) = json.get("item") {
                            let output_index = json
                                .get("output_index")
                                .and_then(|v| v.as_u64())
                                .map(|v| v as usize);
                            match item.get("type").and_then(|v| v.as_str()) {
                                Some("message") => {
                                    let text =
                                        extract_text(item.get("content").unwrap_or(&Value::Null));
                                    merge_response_text_done(&mut response, &text);
                                }
                                Some("function_call") => {
                                    let item_id =
                                        item.get("id").and_then(|v| v.as_str()).map(str::to_string);
                                    let index = ensure_stream_tool_call_index(
                                        &mut response,
                                        &mut output_index_to_tool_call,
                                        &mut item_id_to_tool_call,
                                        output_index,
                                        item_id.as_deref(),
                                    );

                                    if let Some(id) = item
                                        .get("call_id")
                                        .or_else(|| item.get("id"))
                                        .and_then(|v| v.as_str())
                                    {
                                        response.tool_calls[index].id = id.to_string();
                                    }
                                    if let Some(name) = item.get("name").and_then(|v| v.as_str()) {
                                        response.tool_calls[index].name = name.to_string();
                                    }
                                    if let Some(arguments) =
                                        item.get("arguments").and_then(|v| v.as_str())
                                    {
                                        merge_tool_call_arguments_done(
                                            &mut response.tool_calls[index],
                                            arguments,
                                        );
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    Some("response.function_call_arguments.delta") => {
                        let output_index = json
                            .get("output_index")
                            .and_then(|v| v.as_u64())
                            .map(|v| v as usize);
                        let item_id = json.get("item_id").and_then(|v| v.as_str());
                        let index = ensure_stream_tool_call_index(
                            &mut response,
                            &mut output_index_to_tool_call,
                            &mut item_id_to_tool_call,
                            output_index,
                            item_id,
                        );
                        if let Some(call_id) = json.get("call_id").and_then(|v| v.as_str()) {
                            response.tool_calls[index].id = call_id.to_string();
                        }
                        if let Some(delta) = json.get("delta").and_then(|v| v.as_str()) {
                            response.tool_calls[index].arguments.push_str(delta);
                        }
                    }
                    Some("response.function_call_arguments.done") => {
                        let output_index = json
                            .get("output_index")
                            .and_then(|v| v.as_u64())
                            .map(|v| v as usize);
                        let item_id = json.get("item_id").and_then(|v| v.as_str());
                        let index = ensure_stream_tool_call_index(
                            &mut response,
                            &mut output_index_to_tool_call,
                            &mut item_id_to_tool_call,
                            output_index,
                            item_id,
                        );
                        if let Some(call_id) = json.get("call_id").and_then(|v| v.as_str()) {
                            response.tool_calls[index].id = call_id.to_string();
                        }
                        if let Some(arguments) = json.get("arguments").and_then(|v| v.as_str()) {
                            merge_tool_call_arguments_done(
                                &mut response.tool_calls[index],
                                arguments,
                            );
                        }
                    }
                    Some("response.completed") => {
                        if let Some(inner) = json.get("response") {
                            let parsed = parse_responses_response(inner);
                            merge_generic_response(&mut response, parsed);
                        }
                    }
                    _ => {}
                }
                parse_token_usage_by_api(&json.to_string().into_bytes(), api, &mut response.usage);
            }
        }
        ApiFormat::GeminiGenerateContent => {
            return Err("Gemini 响应暂不支持转换".to_string());
        }
    }

    if response.id.is_empty() {
        response.id = format!("ccg_{}", Uuid::new_v4().simple());
    }
    Ok(response)
}

fn serialize_streaming_response(api: ApiFormat, response: &GenericResponse) -> Vec<u8> {
    match api {
        ApiFormat::AnthropicMessages => serialize_anthropic_stream(response),
        ApiFormat::OpenAiChatCompletions => serialize_chat_stream(response),
        ApiFormat::OpenAiResponses => serialize_responses_stream(response),
        ApiFormat::GeminiGenerateContent => vec![],
    }
}

fn serialize_anthropic_stream(response: &GenericResponse) -> Vec<u8> {
    let mut out = String::new();
    push_sse_event(
        &mut out,
        "message_start",
        &json!({
            "type": "message_start",
            "message": {
                "id": non_empty_or_generated(&response.id, "msg"),
                "type": "message",
                "role": "assistant",
                "model": response.model.clone().unwrap_or_else(|| "ccg-gateway".to_string()),
                "content": [],
                "stop_reason": null,
                "stop_sequence": null,
                "usage": {
                    "input_tokens": response.usage.input_tokens,
                    "output_tokens": 0
                }
            }
        }),
    );
    if !response.text.is_empty() {
        push_sse_event(
            &mut out,
            "content_block_start",
            &json!({"type":"content_block_start","index":0,"content_block":{"type":"text","text":""}}),
        );
        push_sse_event(
            &mut out,
            "content_block_delta",
            &json!({"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":response.text}}),
        );
        push_sse_event(
            &mut out,
            "content_block_stop",
            &json!({"type":"content_block_stop","index":0}),
        );
    }
    push_sse_event(
        &mut out,
        "message_delta",
        &json!({"type":"message_delta","delta":{"stop_reason":anthropic_finish_reason(response),"stop_sequence":null},"usage":{"output_tokens":response.usage.output_tokens}}),
    );
    push_sse_event(&mut out, "message_stop", &json!({"type":"message_stop"}));
    out.into_bytes()
}

fn serialize_chat_stream(response: &GenericResponse) -> Vec<u8> {
    let mut out = String::new();
    push_chat_stream_chunk(
        &mut out,
        &json!({
            "id": non_empty_or_generated(&response.id, "chatcmpl"),
            "object": "chat.completion.chunk",
            "created": response.created_at,
            "model": response.model.clone().unwrap_or_else(|| "ccg-gateway".to_string()),
            "choices": [{
                "index": 0,
                "delta": {
                    "role": "assistant",
                    "content": response.text
                },
                "finish_reason": null
            }]
        }),
    );
    push_chat_stream_chunk(
        &mut out,
        &json!({
            "id": non_empty_or_generated(&response.id, "chatcmpl"),
            "object": "chat.completion.chunk",
            "created": response.created_at,
            "model": response.model.clone().unwrap_or_else(|| "ccg-gateway".to_string()),
            "choices": [{
                "index": 0,
                "delta": {},
                "finish_reason": chat_finish_reason(response)
            }]
        }),
    );
    out.push_str("data: [DONE]\n\n");
    out.into_bytes()
}

fn serialize_responses_stream(response: &GenericResponse) -> Vec<u8> {
    let mut out = String::new();
    let response_id = non_empty_or_generated(&response.id, "resp");
    let model = response
        .model
        .clone()
        .unwrap_or_else(|| "ccg-gateway".to_string());
    let output_items = build_responses_output_items(response);

    push_sse_event(
        &mut out,
        "response.created",
        &json!({
            "type":"response.created",
            "response":{
                "id":response_id,
                "object":"response",
                "created_at":response.created_at,
                "model":model,
                "status":"in_progress",
                "output":[]
            }
        }),
    );

    for item in &output_items {
        let output_index = item.output_index();
        push_sse_event(
            &mut out,
            "response.output_item.added",
            &json!({
                "type":"response.output_item.added",
                "response_id":response_id,
                "output_index":output_index,
                "item":item.added_item()
            }),
        );

        match item {
            ResponsesOutputItem::Text { item_id, text, .. } => {
                push_sse_event(
                    &mut out,
                    "response.content_part.added",
                    &json!({
                        "type":"response.content_part.added",
                        "response_id":response_id,
                        "output_index":output_index,
                        "item_id":item_id,
                        "content_index":0,
                        "part":{"type":"output_text","text":""}
                    }),
                );
                push_sse_event(
                    &mut out,
                    "response.output_text.delta",
                    &json!({
                        "type":"response.output_text.delta",
                        "response_id":response_id,
                        "output_index":output_index,
                        "item_id":item_id,
                        "content_index":0,
                        "delta":text
                    }),
                );
                push_sse_event(
                    &mut out,
                    "response.output_text.done",
                    &json!({
                        "type":"response.output_text.done",
                        "response_id":response_id,
                        "output_index":output_index,
                        "item_id":item_id,
                        "content_index":0,
                        "text":text
                    }),
                );
                push_sse_event(
                    &mut out,
                    "response.content_part.done",
                    &json!({
                        "type":"response.content_part.done",
                        "response_id":response_id,
                        "output_index":output_index,
                        "item_id":item_id,
                        "content_index":0,
                        "part":{"type":"output_text","text":text}
                    }),
                );
            }
            ResponsesOutputItem::FunctionCall {
                item_id,
                call_id,
                arguments,
                ..
            } => {
                if !arguments.is_empty() {
                    push_sse_event(
                        &mut out,
                        "response.function_call_arguments.delta",
                        &json!({
                            "type":"response.function_call_arguments.delta",
                            "response_id":response_id,
                            "output_index":output_index,
                            "item_id":item_id,
                            "call_id":call_id,
                            "delta":arguments
                        }),
                    );
                }
                push_sse_event(
                    &mut out,
                    "response.function_call_arguments.done",
                    &json!({
                        "type":"response.function_call_arguments.done",
                        "response_id":response_id,
                        "output_index":output_index,
                        "item_id":item_id,
                        "call_id":call_id,
                        "arguments":arguments
                    }),
                );
            }
        }

        push_sse_event(
            &mut out,
            "response.output_item.done",
            &json!({
                "type":"response.output_item.done",
                "response_id":response_id,
                "output_index":output_index,
                "item":item.completed_item()
            }),
        );
    }

    push_sse_event(
        &mut out,
        "response.completed",
        &json!({
            "type":"response.completed",
            "response":serialize_responses_response_payload(response, &response_id, &model, &output_items)
        }),
    );
    out.into_bytes()
}

fn anthropic_blocks(content: Option<&Value>) -> Vec<Value> {
    match content {
        Some(Value::Array(items)) => items.clone(),
        Some(Value::String(text)) if !text.is_empty() => {
            vec![json!({"type": "text", "text": text})]
        }
        _ => vec![],
    }
}

fn extract_anthropic_system_text(system: Option<&Value>) -> Option<String> {
    let text = extract_text(system.unwrap_or(&Value::Null));
    if text.is_empty() {
        None
    } else {
        Some(text)
    }
}

fn extract_text(value: &Value) -> String {
    match value {
        Value::Null => String::new(),
        Value::String(text) => text.to_string(),
        Value::Array(items) => items
            .iter()
            .filter_map(|item| {
                item.get("text")
                    .and_then(|v| v.as_str())
                    .map(|v| v.to_string())
                    .or_else(|| item.as_str().map(|v| v.to_string()))
            })
            .collect::<Vec<_>>()
            .join(""),
        Value::Object(map) => map
            .get("text")
            .and_then(|v| v.as_str())
            .map(|v| v.to_string())
            .unwrap_or_else(|| serde_json::to_string(map).unwrap_or_default()),
        _ => value.to_string(),
    }
}

fn extract_chat_text(value: &Value) -> String {
    let content = extract_text(value.get("content").unwrap_or(&Value::Null));
    if !content.is_empty() {
        return content;
    }

    extract_text(value.get("reasoning_content").unwrap_or(&Value::Null))
}

fn parse_responses_tool_definition(tool: &Value) -> Option<GenericToolDefinition> {
    let tool_type = tool
        .get("type")
        .and_then(|v| v.as_str())
        .unwrap_or("function");
    match tool_type {
        "function" | "custom" => {
            let name = tool
                .get("name")
                .and_then(|v| v.as_str())
                .map(str::trim)
                .filter(|name| !name.is_empty())?
                .to_string();
            Some(GenericToolDefinition {
                name,
                description: tool
                    .get("description")
                    .and_then(|v| v.as_str())
                    .map(|v| v.to_string()),
                parameters: tool.get("parameters").cloned().unwrap_or_else(|| json!({})),
            })
        }
        _ => None,
    }
}

fn convert_tool_choice_to_anthropic(choice: &Value) -> Option<Value> {
    if let Some(text) = choice.as_str() {
        return Some(match text {
            "required" => json!({"type":"any"}),
            "auto" => json!({"type":"auto"}),
            _ => Value::String(text.to_string()),
        });
    }
    Some(choice.clone())
}

fn convert_tool_choice_to_chat(choice: &Value) -> Option<Value> {
    Some(choice.clone())
}
fn convert_tool_choice_to_responses(choice: &Value) -> Option<Value> {
    Some(choice.clone())
}

fn parse_arguments_json(arguments: &str) -> Value {
    if arguments.trim().is_empty() {
        json!({})
    } else {
        serde_json::from_str(arguments).unwrap_or_else(|_| json!({"raw": arguments}))
    }
}

fn value_to_argument_string(value: &Value) -> String {
    match value {
        Value::String(text) => text.to_string(),
        _ => serde_json::to_string(value).unwrap_or_else(|_| "{}".to_string()),
    }
}

fn push_anthropic_block(messages: &mut Vec<Value>, role: &str, block: Value) {
    if let Some(last) = messages.last_mut() {
        if last.get("role").and_then(|v| v.as_str()) == Some(role) {
            if let Some(content) = last.get_mut("content").and_then(|v| v.as_array_mut()) {
                content.push(block);
                return;
            }
        }
    }
    messages.push(json!({"role": role, "content": [block]}));
}

fn flush_assistant_message(messages: &mut Vec<Value>, current: &mut Option<(String, Vec<Value>)>) {
    if let Some((content, tool_calls)) = current.take() {
        let content_value = if content.is_empty() {
            Value::Null
        } else {
            Value::String(content)
        };
        let mut message = json!({"role":"assistant","content":content_value});
        if !tool_calls.is_empty() {
            message["tool_calls"] = Value::Array(tool_calls);
        }
        messages.push(message);
    }
}

fn parse_sse_frames(body: &[u8]) -> Vec<String> {
    let mut frames = Vec::new();
    let mut current = Vec::new();

    for raw_line in String::from_utf8_lossy(body).lines() {
        let line = raw_line.trim_end_matches('\r');
        if line.is_empty() {
            if !current.is_empty() {
                frames.push(current.join("\n"));
                current.clear();
            }
            continue;
        }

        if line.starts_with(':') {
            continue;
        }

        if let Some(data) = line.strip_prefix("data:") {
            current.push(data.trim_start().to_string());
        }
    }

    if !current.is_empty() {
        frames.push(current.join("\n"));
    }

    frames
}

fn is_error_response_payload(json: &Value) -> bool {
    json.get("error").is_some()
}

fn extract_error_message(json: &Value) -> String {
    let message = json
        .get("error")
        .and_then(|error| {
            error.as_str().map(|v| v.to_string()).or_else(|| {
                error
                    .get("message")
                    .and_then(|v| v.as_str())
                    .map(|v| v.to_string())
            })
        })
        .or_else(|| {
            json.get("message")
                .and_then(|v| v.as_str())
                .map(|v| v.to_string())
        })
        .unwrap_or_else(|| "未知错误".to_string());

    format!("上游返回错误响应: {}", message)
}

fn merge_response_output_item(response: &mut GenericResponse, item: &Value) {
    merge_response_output_item_with_index(response, item, None);
}

fn merge_response_output_item_with_index(
    response: &mut GenericResponse,
    item: &Value,
    output_index: Option<usize>,
) {
    match item.get("type").and_then(|v| v.as_str()) {
        Some("message") => {
            let text = extract_text(item.get("content").unwrap_or(&Value::Null));
            merge_response_text_done(response, &text);
        }
        Some("function_call") => {
            let index = output_index.unwrap_or(response.tool_calls.len());
            while response.tool_calls.len() <= index {
                response.tool_calls.push(GenericToolCall::default());
            }

            if let Some(id) = item
                .get("call_id")
                .or_else(|| item.get("id"))
                .and_then(|v| v.as_str())
            {
                response.tool_calls[index].id = id.to_string();
            }
            if let Some(name) = item.get("name").and_then(|v| v.as_str()) {
                response.tool_calls[index].name = name.to_string();
            }
            if let Some(arguments) = item.get("arguments").and_then(|v| v.as_str()) {
                merge_tool_call_arguments_done(&mut response.tool_calls[index], arguments);
            }
        }
        _ => {}
    }
}

fn merge_response_text_done(response: &mut GenericResponse, text: &str) {
    if text.is_empty() {
        return;
    }

    if response.text.is_empty()
        || (text.len() >= response.text.len() && text.starts_with(&response.text))
    {
        response.text = text.to_string();
    }
}

fn merge_tool_call_arguments_done(tool_call: &mut GenericToolCall, arguments: &str) {
    if arguments.is_empty() {
        return;
    }

    if tool_call.arguments.is_empty()
        || (arguments.len() >= tool_call.arguments.len()
            && arguments.starts_with(&tool_call.arguments))
    {
        tool_call.arguments = arguments.to_string();
    }
}

fn ensure_stream_tool_call_index(
    response: &mut GenericResponse,
    output_index_to_tool_call: &mut std::collections::HashMap<usize, usize>,
    item_id_to_tool_call: &mut std::collections::HashMap<String, usize>,
    output_index: Option<usize>,
    item_id: Option<&str>,
) -> usize {
    if let Some(item_id) = item_id {
        if let Some(index) = item_id_to_tool_call.get(item_id) {
            return *index;
        }
    }
    if let Some(output_index) = output_index {
        if let Some(index) = output_index_to_tool_call.get(&output_index) {
            return *index;
        }
    }

    let index = response.tool_calls.len();
    response.tool_calls.push(GenericToolCall::default());

    if let Some(output_index) = output_index {
        output_index_to_tool_call.insert(output_index, index);
    }
    if let Some(item_id) = item_id {
        item_id_to_tool_call.insert(item_id.to_string(), index);
    }

    index
}

fn build_responses_output_items(response: &GenericResponse) -> Vec<ResponsesOutputItem> {
    let mut output_items = Vec::new();
    let mut output_index = 0usize;

    if !response.text.is_empty() {
        output_items.push(ResponsesOutputItem::Text {
            output_index,
            item_id: format!("msg_{}", Uuid::new_v4().simple()),
            text: response.text.clone(),
        });
        output_index += 1;
    }

    for tool_call in &response.tool_calls {
        output_items.push(ResponsesOutputItem::FunctionCall {
            output_index,
            item_id: format!("fc_{}", Uuid::new_v4().simple()),
            call_id: non_empty_or_generated(&tool_call.id, "call"),
            name: tool_call.name.clone(),
            arguments: tool_call.arguments.clone(),
        });
        output_index += 1;
    }

    output_items
}

fn serialize_responses_response_payload(
    response: &GenericResponse,
    response_id: &str,
    model: &str,
    output_items: &[ResponsesOutputItem],
) -> Value {
    json!({
        "id": response_id,
        "object": "response",
        "created_at": response.created_at,
        "status": "completed",
        "model": model,
        "output": output_items
            .iter()
            .map(ResponsesOutputItem::completed_item)
            .collect::<Vec<_>>(),
        "usage": {
            "input_tokens": response.usage.input_tokens,
            "output_tokens": response.usage.output_tokens,
            "total_tokens": response.usage.input_tokens + response.usage.output_tokens
        }
    })
}

fn merge_generic_response(target: &mut GenericResponse, source: GenericResponse) {
    if target.id.is_empty() {
        target.id = source.id;
    }
    if target.model.is_none() {
        target.model = source.model;
    }
    if target.text.is_empty() {
        target.text = source.text;
    }
    if target.tool_calls.is_empty() {
        target.tool_calls = source.tool_calls;
    }
    if target.finish_reason.is_none() {
        target.finish_reason = source.finish_reason;
    }
    if target.usage.input_tokens == 0 {
        target.usage.input_tokens = source.usage.input_tokens;
    }
    if target.usage.output_tokens == 0 {
        target.usage.output_tokens = source.usage.output_tokens;
    }
}

fn anthropic_finish_reason(response: &GenericResponse) -> &'static str {
    if !response.tool_calls.is_empty() {
        "tool_use"
    } else {
        "end_turn"
    }
}
fn chat_finish_reason(response: &GenericResponse) -> &'static str {
    if !response.tool_calls.is_empty() {
        "tool_calls"
    } else {
        "stop"
    }
}

fn non_empty_or_generated(value: &str, prefix: &str) -> String {
    if value.is_empty() {
        format!("{}_{}", prefix, Uuid::new_v4().simple())
    } else {
        value.to_string()
    }
}

fn push_sse_event(out: &mut String, event: &str, payload: &Value) {
    out.push_str("event: ");
    out.push_str(event);
    out.push('\n');
    out.push_str("data: ");
    out.push_str(&serde_json::to_string(payload).unwrap_or_else(|_| "{}".to_string()));
    out.push_str("\n\n");
}

fn push_chat_stream_chunk(out: &mut String, payload: &Value) {
    out.push_str("data: ");
    out.push_str(&serde_json::to_string(payload).unwrap_or_else(|_| "{}".to_string()));
    out.push_str("\n\n");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::models::DEFAULT_GATEWAY_MAX_TOKENS;

    #[test]
    fn converts_anthropic_request_to_chat_request() {
        let source = json!({"model":"claude-3-7-sonnet","stream":true,"max_tokens":1024,"system":[{"type":"text","text":"你是助手"}],"messages":[{"role":"user","content":[{"type":"text","text":"你好"}]},{"role":"assistant","content":[{"type":"tool_use","id":"call_1","name":"lookup","input":{"q":"rust"}}]},{"role":"user","content":[{"type":"tool_result","tool_use_id":"call_1","content":"done"}]}]});
        let converted = transform_request(
            ApiFormat::AnthropicMessages,
            ApiFormat::OpenAiChatCompletions,
            "/v1/messages",
            source.to_string().as_bytes(),
            DEFAULT_GATEWAY_MAX_TOKENS,
        )
        .expect("conversion should succeed");
        let converted_json: Value = serde_json::from_slice(&converted.body).expect("json");
        assert_eq!(converted.path, "/v1/chat/completions");
        assert_eq!(converted_json["max_tokens"], 1024);
        assert_eq!(converted_json["messages"][0]["role"], "system");
        assert_eq!(converted_json["messages"][1]["role"], "user");
        assert_eq!(
            converted_json["messages"][2]["tool_calls"][0]["function"]["name"],
            "lookup"
        );
        assert_eq!(converted_json["messages"][3]["role"], "tool");
    }

    #[test]
    fn adds_default_max_tokens_when_converting_responses_request_to_chat_request() {
        let source = json!({
            "model": "gpt-5.4",
            "stream": true,
            "input": "hello"
        });

        let converted = transform_request(
            ApiFormat::OpenAiResponses,
            ApiFormat::OpenAiChatCompletions,
            "/v1/responses",
            source.to_string().as_bytes(),
            DEFAULT_GATEWAY_MAX_TOKENS,
        )
        .expect("request conversion should succeed");
        let converted_json: Value = serde_json::from_slice(&converted.body).expect("json");

        assert_eq!(converted_json["max_tokens"], DEFAULT_GATEWAY_MAX_TOKENS);
    }

    #[test]
    fn adds_missing_max_tokens_for_passthrough_chat_request() {
        let source = json!({
            "model": "gpt-5.4",
            "stream": true,
            "max_completion_tokens": 2048,
            "messages": [{
                "role": "user",
                "content": "hello"
            }]
        });

        let converted = transform_request(
            ApiFormat::OpenAiChatCompletions,
            ApiFormat::OpenAiChatCompletions,
            "/v1/chat/completions",
            source.to_string().as_bytes(),
            DEFAULT_GATEWAY_MAX_TOKENS,
        )
        .expect("passthrough request should succeed");
        let converted_json: Value = serde_json::from_slice(&converted.body).expect("json");

        assert_eq!(converted_json["max_tokens"], 2048);
        assert_eq!(converted_json["max_completion_tokens"], 2048);
    }

    #[test]
    fn converts_chat_response_to_anthropic_response() {
        let source = json!({"id":"chatcmpl_123","object":"chat.completion","created":123,"model":"gpt-4.1","choices":[{"index":0,"message":{"role":"assistant","content":"已完成"},"finish_reason":"stop"}],"usage":{"prompt_tokens":12,"completion_tokens":34}});
        let converted = transform_response_body(
            ApiFormat::OpenAiChatCompletions,
            ApiFormat::AnthropicMessages,
            source.to_string().as_bytes(),
        )
        .expect("response conversion should succeed");
        let converted_json: Value = serde_json::from_slice(&converted).expect("json");
        assert_eq!(converted_json["content"][0]["type"], "text");
        assert_eq!(converted_json["usage"]["input_tokens"], 12);
        assert_eq!(converted_json["usage"]["output_tokens"], 34);
    }

    #[test]
    fn converts_responses_stream_to_chat_stream() {
        let source = concat!(
            "event: response.created\n",
            "data: {\"type\":\"response.created\",\"response\":{\"id\":\"resp_1\",\"model\":\"gpt-4.1\"}}\n\n",
            "event: response.output_text.delta\n",
            "data: {\"type\":\"response.output_text.delta\",\"output_index\":0,\"content_index\":0,\"delta\":\"Hello\"}\n\n",
            "event: response.completed\n",
            "data: {\"type\":\"response.completed\",\"response\":{\"id\":\"resp_1\",\"model\":\"gpt-4.1\",\"status\":\"completed\",\"output\":[{\"type\":\"message\",\"role\":\"assistant\",\"content\":[{\"type\":\"output_text\",\"text\":\"Hello\"}]}],\"usage\":{\"input_tokens\":1,\"output_tokens\":2}}}\n\n"
        );
        let converted = transform_streaming_response(
            ApiFormat::OpenAiResponses,
            ApiFormat::OpenAiChatCompletions,
            source.as_bytes(),
        )
        .expect("stream conversion should succeed");
        let text = String::from_utf8(converted).expect("utf8");
        assert!(text.contains("\"chat.completion.chunk\""));
        assert!(text.contains("\"content\":\"Hello\""));
        assert!(text.contains("data: [DONE]"));
    }

    #[test]
    fn converts_chat_stream_with_crlf_to_responses_stream() {
        let source = concat!(
            "data: {\"id\":\"chatcmpl_1\",\"object\":\"chat.completion.chunk\",\"created\":123,\"model\":\"gpt-4.1\",\"choices\":[{\"index\":0,\"delta\":{\"role\":\"assistant\",\"content\":\"Hello\"},\"finish_reason\":null}]}\r\n",
            "\r\n",
            "data: {\"id\":\"chatcmpl_1\",\"object\":\"chat.completion.chunk\",\"created\":123,\"model\":\"gpt-4.1\",\"choices\":[{\"index\":0,\"delta\":{},\"finish_reason\":\"stop\"}],\"usage\":{\"prompt_tokens\":1,\"completion_tokens\":2,\"total_tokens\":3}}\r\n",
            "\r\n",
            "data: [DONE]\r\n",
            "\r\n"
        );
        let converted = transform_streaming_response(
            ApiFormat::OpenAiChatCompletions,
            ApiFormat::OpenAiResponses,
            source.as_bytes(),
        )
        .expect("crlf stream conversion should succeed");
        let text = String::from_utf8(converted).expect("utf8");
        assert!(text.contains("\"type\":\"response.output_item.added\""));
        assert!(text.contains("\"type\":\"response.content_part.added\""));
        assert!(text.contains("\"type\":\"response.output_text.delta\""));
        assert!(text.contains("\"type\":\"response.output_text.done\""));
        assert!(text.contains("\"type\":\"response.output_item.done\""));
        assert!(text.contains("\"delta\":\"Hello\""));
        assert!(text.contains("\"input_tokens\":1"));
        assert!(text.contains("\"output_tokens\":2"));
    }

    #[test]
    fn converts_chat_json_response_to_responses_stream() {
        let source = json!({
            "id": "chatcmpl_123",
            "object": "chat.completion",
            "created": 123,
            "model": "gpt-4.1",
            "choices": [{
                "index": 0,
                "message": {
                    "role": "assistant",
                    "content": "Hello"
                },
                "finish_reason": "stop"
            }],
            "usage": {
                "prompt_tokens": 1,
                "completion_tokens": 2,
                "total_tokens": 3
            }
        });
        let converted = transform_streaming_response(
            ApiFormat::OpenAiChatCompletions,
            ApiFormat::OpenAiResponses,
            source.to_string().as_bytes(),
        )
        .expect("json response should be converted to responses stream");
        let text = String::from_utf8(converted).expect("utf8");
        assert!(text.contains("\"type\":\"response.output_item.added\""));
        assert!(text.contains("\"type\":\"response.output_text.delta\""));
        assert!(text.contains("\"type\":\"response.output_text.done\""));
        assert!(text.contains("\"delta\":\"Hello\""));
        assert!(text.contains("\"input_tokens\":1"));
        assert!(text.contains("\"output_tokens\":2"));
    }

    #[test]
    fn converts_chat_reasoning_content_stream_to_responses_text() {
        let source = concat!(
            "data: {\"id\":\"chatcmpl_1\",\"object\":\"chat.completion.chunk\",\"created\":123,\"model\":\"glm-4.7\",\"choices\":[{\"index\":0,\"delta\":{\"role\":\"assistant\",\"content\":null,\"reasoning_content\":\"用户\"},\"finish_reason\":null}]}\n\n",
            "data: {\"id\":\"chatcmpl_1\",\"object\":\"chat.completion.chunk\",\"created\":123,\"model\":\"glm-4.7\",\"choices\":[{\"index\":0,\"delta\":{\"content\":null,\"reasoning_content\":\"hello\"},\"finish_reason\":\"stop\"}],\"usage\":{\"prompt_tokens\":1,\"completion_tokens\":2,\"total_tokens\":3}}\n\n",
            "data: [DONE]\n\n"
        );
        let converted = transform_streaming_response(
            ApiFormat::OpenAiChatCompletions,
            ApiFormat::OpenAiResponses,
            source.as_bytes(),
        )
        .expect("reasoning_content stream conversion should succeed");
        let text = String::from_utf8(converted).expect("utf8");
        assert!(text.contains("\"type\":\"response.output_item.added\""));
        assert!(text.contains("\"type\":\"response.output_text.delta\""));
        assert!(text.contains("\"type\":\"response.output_text.done\""));
        assert!(text.contains("\"delta\":\"用户hello\""));
        assert!(text.contains("\"input_tokens\":1"));
        assert!(text.contains("\"output_tokens\":2"));
    }

    #[test]
    fn converts_chat_stream_with_tool_call_to_responses_events() {
        let source = concat!(
            "data: {\"id\":\"chatcmpl_1\",\"object\":\"chat.completion.chunk\",\"created\":123,\"model\":\"glm-4.7\",\"choices\":[{\"index\":0,\"delta\":{\"role\":\"assistant\",\"content\":null,\"reasoning_content\":\"先\"},\"finish_reason\":null}]}\n\n",
            "data: {\"id\":\"chatcmpl_1\",\"object\":\"chat.completion.chunk\",\"created\":123,\"model\":\"glm-4.7\",\"choices\":[{\"index\":0,\"delta\":{\"content\":\"查看日志\",\"reasoning_content\":null},\"finish_reason\":null}]}\n\n",
            "data: {\"id\":\"chatcmpl_1\",\"object\":\"chat.completion.chunk\",\"created\":123,\"model\":\"glm-4.7\",\"choices\":[{\"index\":0,\"delta\":{\"content\":\"\",\"tool_calls\":[{\"index\":0,\"id\":\"call_1\",\"type\":\"function\",\"function\":{\"name\":\"shell_command\",\"arguments\":\"{\\\"command\\\":\\\"Get-ChildItem -Force\\\"}\"}}],\"reasoning_content\":null},\"finish_reason\":\"tool_calls\"}],\"usage\":{\"prompt_tokens\":1,\"completion_tokens\":2,\"total_tokens\":3}}\n\n",
            "data: [DONE]\n\n"
        );

        let converted = transform_streaming_response(
            ApiFormat::OpenAiChatCompletions,
            ApiFormat::OpenAiResponses,
            source.as_bytes(),
        )
        .expect("tool call stream conversion should succeed");
        let text = String::from_utf8(converted.clone()).expect("utf8");
        assert!(text.contains("\"type\":\"response.output_item.added\""));
        assert!(text.contains("\"type\":\"response.content_part.added\""));
        assert!(text.contains("\"type\":\"response.output_text.delta\""));
        assert!(text.contains("\"type\":\"response.output_text.done\""));
        assert!(text.contains("\"type\":\"response.function_call_arguments.delta\""));
        assert!(text.contains("\"type\":\"response.function_call_arguments.done\""));
        assert!(text.contains("\"delta\":\"先查看日志\""));
        assert!(text.contains("\"arguments\":\"{\\\"command\\\":\\\"Get-ChildItem -Force\\\"}\""));

        let parsed = parse_streaming_response(ApiFormat::OpenAiResponses, &converted)
            .expect("generated responses stream should be parseable");
        assert_eq!(parsed.text, "先查看日志");
        assert_eq!(parsed.tool_calls.len(), 1);
        assert_eq!(parsed.tool_calls[0].id, "call_1");
        assert_eq!(parsed.tool_calls[0].name, "shell_command");
        assert_eq!(
            parsed.tool_calls[0].arguments,
            "{\"command\":\"Get-ChildItem -Force\"}"
        );
    }

    #[test]
    fn converts_responses_request_to_chat_without_unnamed_tools() {
        let source = json!({
            "model": "gpt-5.4",
            "stream": true,
            "input": "hello",
            "tool_choice": "auto",
            "tools": [
                {
                    "type": "function",
                    "name": "shell_command",
                    "description": "run shell",
                    "parameters": {"type":"object","properties":{"command":{"type":"string"}},"required":["command"]}
                },
                {
                    "type": "custom",
                    "name": "apply_patch",
                    "description": "patch file"
                },
                {
                    "type": "web_search",
                    "external_web_access": true
                }
            ]
        });

        let converted = transform_request(
            ApiFormat::OpenAiResponses,
            ApiFormat::OpenAiChatCompletions,
            "/v1/responses",
            source.to_string().as_bytes(),
            DEFAULT_GATEWAY_MAX_TOKENS,
        )
        .expect("request conversion should succeed");
        let converted_json: Value = serde_json::from_slice(&converted.body).expect("json");
        let tools = converted_json["tools"].as_array().expect("tools array");
        assert_eq!(tools.len(), 2);
        assert_eq!(tools[0]["function"]["name"], "shell_command");
        assert_eq!(tools[1]["function"]["name"], "apply_patch");
        assert!(tools
            .iter()
            .all(|tool| tool["function"]["name"].as_str().unwrap_or("").len() > 0));
    }
}
