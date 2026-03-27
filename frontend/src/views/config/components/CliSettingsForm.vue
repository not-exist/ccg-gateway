<template>
  <div>
    <div style="margin-bottom: 24px;">
      <div style="font-size: 14px; color: #475569; font-weight: 500; margin-bottom: 10px;">配置目录</div>
      <div style="display: flex; width: 100%; gap: 8px;">
        <input type="text" v-model="form.config_dir" class="c-input" style="flex: 1; outline: none;" placeholder="CLI 配置目录路径">
        <button class="b-button-outline" @click="handleResetDir">重置</button>
      </div>
      <div style="font-size: 11px; color: #94a3b8; margin-top: 6px;">{{ configDirTip }}</div>
    </div>
    
    <div>
      <div style="font-size: 14px; color: #475569; font-weight: 500; margin-bottom: 10px;">默认配置</div>
      <textarea 
        v-model="form.default_json_config"
        @blur="validateConfig"
        class="c-textarea"
        :placeholder="placeholder"
      ></textarea>
      <div v-if="validationError" style="font-size: 12px; color: #ef4444; margin-top: 6px;">{{ validationError }}</div>
      <div style="font-size: 12px; color: #94a3b8; margin-top: 6px;">{{ tip }}</div>
    </div>
    
    <div style="display: flex; gap: 12px; margin-top: 32px; justify-content: flex-end;">
      <button class="b-button-outline" @click="formatJson" :disabled="!isJsonFormat">格式化</button>
      <button class="b-button" @click="handleSave">保存</button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, computed } from 'vue'
import { notify } from '@/utils/notification'
import type { CliSettings } from '@/types/models'
import { validateJson, formatJson as formatJsonUtil } from '@/utils/json'

const props = defineProps<{
  cliType: string
  settings?: CliSettings
}>()

const emit = defineEmits<{
  save: [cliType: string, data: { default_json_config: string; config_dir: string }]
}>()

const form = ref({
  default_json_config: '',
  config_dir: ''
})

const defaultConfigDir = ref('')
const validationError = ref('')

const placeholder = computed(() => {
  switch (props.cliType) {
    case 'codex':
      return `model_reasoning_effort = "high"\nmodel_reasoning_summary = "detailed"`
    case 'claude_code':
      return `{\n  "env": {},\n  "permissions": {}\n}`
    case 'gemini':
      return `{\n  "theme": "dark"\n}`
    default:
      return '{}'
  }
})

const tip = computed(() => {
  switch (props.cliType) {
    case 'codex':
      return '此处配置会合并到 config.toml（TOML 格式）'
    case 'claude_code':
      return '此处配置会合并到 settings.json（JSON 格式）'
    case 'gemini':
      return '此处配置会合并到 settings.json（JSON 格式）'
    default:
      return '此处配置会合并到 CLI 的配置文件中'
  }
})

const isJsonFormat = computed(() => props.cliType === 'claude_code' || props.cliType === 'gemini')
const configDirTip = computed(() => `默认：${defaultConfigDir.value}`)

watch(() => props.settings, (settings) => {
  if (settings) {
    form.value = {
      default_json_config: settings.default_json_config,
      config_dir: settings.config_dir
    }
    defaultConfigDir.value = settings.default_config_dir
  }
}, { immediate: true })

function handleResetDir() {
  form.value.config_dir = defaultConfigDir.value
}

function validateConfig() {
  validationError.value = ''
  const config = form.value.default_json_config.trim()
  if (!config) return true

  if (props.cliType === 'claude_code' || props.cliType === 'gemini') {
    validationError.value = validateJson(config)
    return !validationError.value
  }

  if (props.cliType === 'codex') {
    if (config.includes('{') || (config.includes('[') && config.includes(']') && config.includes(','))) {
      validationError.value = 'TOML 格式错误: 请使用 TOML 格式而非 JSON 格式'
      return false
    }
  }

  return true
}

function formatJson() {
  const result = formatJsonUtil(form.value.default_json_config)
  if (result === form.value.default_json_config) {
    validationError.value = validateJson(form.value.default_json_config)
  } else {
    form.value.default_json_config = result
    validationError.value = ''
  }
}

function handleSave() {
  if (!validateConfig()) {
    notify('配置格式错误，请修正后再保存', 'error')
    return
  }
  emit('save', props.cliType, form.value)
}
</script>

<style scoped>
.b-button { background: #0ea5e9; color: white; border: none; padding: 8px 16px; border-radius: 8px; font-size: 14px; font-weight: 500; cursor: pointer; transition: transform 0.2s, box-shadow 0.2s; outline: none; }
.b-button:hover:not(:disabled) { transform: translateY(-1px); box-shadow: 0 4px 8px rgba(14, 165, 233, 0.3); }
.b-button:disabled { opacity: 0.6; cursor: not-allowed; }

.b-button-outline { background: white; color: #0f172a; border: 1px solid #e2e8f0; padding: 8px 16px; border-radius: 8px; font-size: 14px; font-weight: 500; cursor: pointer; transition: background 0.2s; outline: none; }
.b-button-outline:hover:not(:disabled) { background: #f8fafc; }
.b-button-outline:disabled { opacity: 0.6; cursor: not-allowed; }

.c-input { width: 100%; padding: 10px 14px; border: 1px solid #e2e8f0; border-radius: 8px; font-size: 14px; outline: none; transition: border-color 0.2s, box-shadow 0.2s; }
.c-input:focus { border-color: #0ea5e9; box-shadow: 0 0 0 3px rgba(14, 165, 233, 0.1); }

.c-textarea { 
  width: 100%; height: 180px; padding: 12px; border: 1px solid #e2e8f0; border-radius: 8px; 
  font-family: monospace; font-size: 13px; background: #f8fafc; color: #0f172a; resize: vertical; 
  outline: none; transition: border-color 0.2s, box-shadow 0.2s; 
}
.c-textarea:focus { border-color: #0ea5e9; box-shadow: 0 0 0 3px rgba(14, 165, 233, 0.1); }
</style>
