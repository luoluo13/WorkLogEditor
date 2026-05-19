<script setup lang="ts">
import { NModal, NForm, NFormItem, NInput, NButton, NDivider, NSwitch } from 'naive-ui'
import { useSettingsStore } from '../stores/settings'

defineProps<{
  show: boolean
}>()

const emit = defineEmits(['update:show'])

const settingsStore = useSettingsStore()

const handleClose = () => {
  emit('update:show', false)
}

const handleSave = async () => {
  await settingsStore.saveSettings()
  handleClose()
}
</script>

<template>
  <n-modal
    :show="show"
    preset="card"
    title="应用设置"
    class="max-w-xl"
    @update:show="handleClose"
  >
    <n-form>
      <h3 class="text-lg font-bold mb-4">AI 配置</h3>
      <n-form-item label="DeepSeek API Key">
        <n-input
          v-model:value="settingsStore.apiKey"
          type="password"
          show-password-on="click"
          placeholder="输入您的 API Key"
        />
      </n-form-item>
      <n-form-item label="Base URL">
        <n-input v-model:value="settingsStore.apiEndpoint" placeholder="https://api.deepseek.com/v1" />
      </n-form-item>
      <n-form-item label="模型名称">
        <n-input v-model:value="settingsStore.model" placeholder="deepseek-chat" />
      </n-form-item>

      <n-divider />

      <h3 class="text-lg font-bold mb-4">常规设置</h3>
      <n-form-item label="深色模式">
        <n-switch :value="settingsStore.isDarkMode" @update:value="settingsStore.toggleDarkMode" />
      </n-form-item>
    </n-form>

    <template #footer>
      <div class="flex justify-end space-x-2">
        <n-button @click="handleClose">取消</n-button>
        <n-button type="primary" @click="handleSave">保存</n-button>
      </div>
    </template>
  </n-modal>
</template>
