# 服务商接口类型与协议转换

## 背景

原有实现主要按客户端原始协议透传到上游服务商，导致同一 CLI 请求无法灵活切换到不同供应商支持的接口族。

## 当前设计

- 服务商新增 `api_format` 字段。
- `api_format` 当前支持:
  - `anthropic_messages`
  - `openai_chat_completions`
  - `openai_responses`
  - `gemini_generate_content`
- 网关在转发前根据:
  - 客户端来源协议
  - 服务商目标协议
  - 模型映射结果
  自动生成目标请求体与目标路径。

## 流式转换补充

- `OpenAI Chat Completions -> OpenAI Responses` 的流式转换现在兼容 `LF` 与 `CRLF` 两种 SSE 换行格式。
- 当上游声明为流式但实际返回普通 JSON 成功体时，网关会自动回退为“先解析完整 JSON，再生成目标 SSE”。
- 当跨协议流式请求遇到上游 JSON 错误体时，网关不再伪造空的 SSE 成功事件，而是透传原始错误响应与状态码。
- 对部分非标准 `OpenAI Chat Completions` 供应商，如果流式 chunk 只返回 `reasoning_content` 而不返回 `content`，网关会在转换到 `OpenAI Responses` 时把 `reasoning_content` 作为文本回退，避免客户端看到空回复。
- 当目标接口是 `OpenAI Chat Completions` 时，网关会确保请求体带上 `max_tokens`：
  - 已存在 `max_tokens` 时保持原值。
  - 仅存在 `max_completion_tokens` 时补写同值的 `max_tokens`。
  - 两者都缺失时，使用全局配置中的默认值，初始值为 `256000`。
- 对历史 `Codex` 服务商记录，如果数据库里的 `api_format` 仍为 `NULL`，默认目标接口会继续保持为 `OpenAI Responses`，避免升级后被静默切换到 `/v1/chat/completions`。
- 只有主 completion 路由会参与协议重写；像 `/v1/models`、`/v1/responses/{id}/cancel` 以及附带查询参数的原始辅助路径会直接透传，避免 catch-all 代理错误改写 URL。
- 当 `Codex` 通过 `/responses` 消费 `OpenAI Chat Completions` 流式上游时，网关现在会补齐 `response.output_item.added/done`、`response.content_part.added/done`、`response.output_text.done` 与 `response.function_call_arguments.delta/done` 事件，而不是只发送简化的 `response.output_text.delta + response.completed`，以避免客户端因事件生命周期不完整而显示空回显。
- `OpenAI Responses` 流式解析不再把 `output_index` 直接当作工具调用索引；当前实现会单独维护 `output_index/item_id -> tool_call` 映射，避免前面存在文本输出项时把工具调用解析成空槽位。
- 跨协议流式响应不再等待上游完整结束后一次性转换；当前实现会边接收边解析 SSE 事件，并即时转发给目标协议客户端。
- 当跨协议非流式请求收到上游 `4xx/5xx` 错误时，网关不会再尝试把错误 JSON 当成成功响应结构转换，而是保留原始状态码、响应头与错误体直接返回给客户端。

## 工具转换限制

- `OpenAI Responses` 的工具列表里可能包含 `web_search` 这类没有显式 `name` 的原生工具类型。
- 当目标接口是 `OpenAI Chat Completions` 时，这类工具无法直接映射成合法的 `function` 定义。
- 当前策略是:
  - 保留有名字的 `function/custom` 工具
  - 跳过无名或不兼容的工具项
- 设计目标是优先保证请求体合法，避免上游因空函数名返回 `400 Bad Request`。

## 受影响模块

- `src-tauri/src/api/handlers.rs`
- `src-tauri/src/services/transform.rs`
- `src-tauri/src/services/provider.rs`
- `frontend/src/views/providers/index.vue`
- `src-tauri/src/db/models.rs`
- `src-tauri/src/db/schema_definition.rs`
