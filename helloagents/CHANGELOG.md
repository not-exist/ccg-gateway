# Changelog

## Unreleased

### Added
- 为服务商新增 `api_format` 配置，允许按供应商选择目标接口类型。

### Changed
- 请求代理链路新增内置协议转换，支持 `Anthropic Messages`、`OpenAI Chat Completions` 与 `OpenAI Responses` 之间的请求/响应转换。
- 服务商模型检测功能改为按供应商目标接口发起测试请求。

### Fixed
- 修复 `ProviderResponse` 转换时对 `cli_type` 的所有权冲突，恢复 Rust 编译通过。
- 补齐本地 Windows Rust/MSVC 构建工具链，并完成 `tauri build --no-bundle` 单文件 exe 验证。
- 修复 `Codex -> OpenAI Chat Completions` 跨协议流式转换时的空 `response.completed` 问题，补齐 `CRLF` SSE 解析与普通 JSON 响应回退转换。
- 修复跨协议流式请求在上游返回 JSON 错误体时的错误包装，改为透传原始错误响应。
- 修复 `OpenAI Responses -> Chat Completions` 工具转换会错误生成空函数名的问题，过滤无名/不兼容工具定义，避免上游 `Function name at index N cannot be empty` 校验失败。
- 修复部分 `/chat/completions` 供应商仅返回 `reasoning_content` 导致 Codex 端显示空回复的问题，在 `content` 为空时回退使用 `reasoning_content` 作为可显示文本。
- 恢复 `OpenAI Chat Completions` 请求缺少 `max_tokens` 时的自动补值逻辑，并改为读取全局配置；默认值现为 `256000`，可在应用的基础配置中调整。
- 修复 `OpenAI Chat Completions -> OpenAI Responses` 流式转换事件过于精简导致 Codex 端空回显的问题，补齐 `response.output_item.*`、`response.content_part.*`、`response.output_text.done` 与 `response.function_call_arguments.*` 事件，并修正带文本前导项时的 `output_index -> tool_call` 映射。
- 修复旧版 `Codex` 服务商在 `api_format` 为空时被错误回退到 `OpenAI Chat Completions` 的兼容性回归，恢复默认走 `OpenAI Responses`，避免升级后无感切换上游接口。
- 修复 catch-all 代理对非 completion 路由的错误协议重写，`/v1/models`、`/v1/responses/{id}/cancel` 及带查询参数的原始路径现在会按原样透传。
- 修复跨协议流式响应被整包缓冲后再返回的问题，改为按 SSE 事件增量转换并流式转发，避免长请求超时和大响应内存峰值。
- 修复跨协议非流式请求在上游返回 `4xx/5xx` 错误体时被二次解析成网关 `502` 的问题，现已改为保留原始状态码、响应头与错误体。
- 修复跨协议流式/非流式响应在网关转换失败时的错误收口：流式路径现在会向客户端发送错误 SSE，并把请求按失败写入日志与熔断统计；非流式路径在返回 `502` 前也会先记录失败原因、请求日志与失败计数。
