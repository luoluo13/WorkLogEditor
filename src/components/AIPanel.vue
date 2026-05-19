<script setup lang="ts">
import { ref } from 'vue'
import { NInput, NButton, NScrollbar, NAvatar, useMessage, NPopover, NForm, NFormItem, NIcon } from 'naive-ui'
import { SendOutline, ChatbubbleEllipsesOutline, SettingsOutline, TrashOutline } from '@vicons/ionicons5'
import { invoke } from '@tauri-apps/api/core'
import { useSettingsStore } from '../stores/settings'

const settingsStore = useSettingsStore()
const message = useMessage()
const userInput = ref('')
const messages = ref<{ role: 'user' | 'assistant', content: string }[]>([
  { role: 'assistant', content: '你好！我是你的 AI 助手，有什么可以帮你的吗？' }
])
const loading = ref(false)

const sendMessage = async () => {
  if (!userInput.value.trim() || loading.value) return
  
  if (!settingsStore.apiKey) {
    message.warning('请先配置 API Key')
    return
  }

  const question = userInput.value
  messages.value.push({ role: 'user', content: question })
  userInput.value = ''
  loading.value = true

  try {
    const chatMessages = messages.value.map(m => ({
      role: m.role,
      content: m.content
    }))

    const response = await invoke<string>('ask_ai', { 
      apiKey: settingsStore.apiKey,
      baseUrl: settingsStore.apiEndpoint,
      model: settingsStore.model,
      messages: chatMessages
    })
    messages.value.push({ role: 'assistant', content: response })
  } catch (err: any) {
    console.error(err)
    message.error(`AI 调用失败: ${err}`)
    messages.value.push({ role: 'assistant', content: '抱歉，请求失败。请检查配置或网络。' })
  } finally {
    loading.value = false
  }
}

const clearChat = () => {
  messages.value = [{ role: 'assistant', content: '你好！我是你的 AI 助手，有什么可以帮你的吗？' }]
}
</script>

<template>
  <div class="flex flex-col h-full p-4">
    <div class="font-bold mb-4 flex items-center justify-between">
      <div class="flex items-center">
        <n-avatar round size="small" class="mr-2">
          <ChatbubbleEllipsesOutline />
        </n-avatar>
        AI 助手
      </div>
      <div class="flex space-x-1">
        <n-button quaternary circle size="small" @click="clearChat">
          <template #icon><n-icon><TrashOutline /></n-icon></template>
        </n-button>
        <n-popover trigger="click" placement="bottom-end" :width="300">
          <template #trigger>
            <n-button quaternary circle size="small">
              <template #icon><n-icon><SettingsOutline /></n-icon></template>
            </n-button>
          </template>
          <div class="p-2">
            <h3 class="font-bold mb-4">AI 配置</h3>
            <n-form size="small">
              <n-form-item label="API Key">
                <n-input v-model:value="settingsStore.apiKey" type="password" placeholder="sk-..." @blur="settingsStore.saveSettings" />
              </n-form-item>
              <n-form-item label="Base URL">
                <n-input v-model:value="settingsStore.apiEndpoint" placeholder="https://api.deepseek.com/v1" @blur="settingsStore.saveSettings" />
              </n-form-item>
              <n-form-item label="模型名称">
                <n-input v-model:value="settingsStore.model" placeholder="deepseek-chat" @blur="settingsStore.saveSettings" />
              </n-form-item>
            </n-form>
          </div>
        </n-popover>
      </div>
    </div>
    
    <n-scrollbar class="flex-1 pr-2">
      <div class="space-y-4">
        <div
          v-for="(msg, index) in messages"
          :key="index"
          :class="['flex', msg.role === 'user' ? 'justify-end' : 'justify-start']"
        >
          <div
            :class="[
              'max-w-[85%] p-3 rounded-lg text-sm whitespace-pre-wrap',
              msg.role === 'user' 
                ? 'bg-blue-600 text-white rounded-br-none' 
                : 'bg-gray-100 dark:bg-zinc-800 rounded-bl-none'
            ]"
          >
            {{ msg.content }}
          </div>
        </div>
      </div>
    </n-scrollbar>

    <div class="mt-4 pt-4 border-t border-gray-200 dark:border-zinc-800">
      <n-input
        v-model:value="userInput"
        type="textarea"
        placeholder="输入指令 (如: 总结今天的工作)..."
        :autosize="{ minRows: 2, maxRows: 5 }"
        @keypress.enter.prevent="sendMessage"
      />
      <div class="flex justify-end mt-2">
        <n-button type="primary" :loading="loading" @click="sendMessage">
          <template #icon>
            <SendOutline />
          </template>
          发送
        </n-button>
      </div>
    </div>
  </div>
</template>
