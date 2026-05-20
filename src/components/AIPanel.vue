<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { NInput, NButton, NScrollbar, NAvatar, useMessage, NPopover, NForm, NFormItem, NIcon, NModal, NCard, NSpace, NTag } from 'naive-ui'
import { SendOutline, ChatbubbleEllipsesOutline, SettingsOutline, TrashOutline, BuildOutline, CheckmarkCircleOutline, CloseCircleOutline } from '@vicons/ionicons5'
import { invoke } from '@tauri-apps/api/core'
import { useSettingsStore } from '../stores/settings'
import { useLogStore } from '../stores/log'

const settingsStore = useSettingsStore()
const logStore = useLogStore()
const message = useMessage()

interface ChatMessage {
  role: 'user' | 'assistant' | 'system' | 'tool'
  content: string
  tool_calls?: any[]
  tool_call_id?: string
  name?: string
}

const userInput = ref('')
const messages = ref<ChatMessage[]>([
  { role: 'assistant', content: '你好！我是你的智能日志助手。我可以帮你查找日志、统计工作情况，甚至帮你写日志。' }
])
const loading = ref(false)
const tools = ref<any[]>([])

// Tool confirmation modal
const showConfirmModal = ref(false)
const pendingToolCall = ref<any>(null)
const confirmLoading = ref(false)

onMounted(async () => {
  try {
    tools.value = await invoke('get_all_tools')
  } catch (err) {
    console.error('Failed to fetch tools:', err)
  }
})

const chatWithAI = async (currentMessages: ChatMessage[]) => {
  const provider = settingsStore.provider
  const apiKey = provider === 'deepseek' ? settingsStore.apiKey : ''
  const baseUrl = provider === 'deepseek' ? settingsStore.apiEndpoint : settingsStore.ollamaEndpoint
  const model = provider === 'deepseek' ? settingsStore.model : settingsStore.ollamaModel

  const systemPrompt = {
    role: 'system',
    content: `你是一个专业的工作日志助手。你的目标是帮助用户记录、查询和总结工作日志。
当前日期: ${new Date().toLocaleDateString()}
你的规则：
1. 必须优先调用工具获取真实日志数据，严禁虚构日志内容。
2. 当用户要求总结时，先获取相关日期的日志，再进行总结。
3. 当用户要求创建或修改日志时，调用对应工具，并等待用户确认。
4. 回答风格应专业、简洁、结构化，并积极提供建议。`
  }

  const allMessages = [systemPrompt, ...currentMessages]

  return await invoke<any>('ask_ai_with_tools', {
    apiKey,
    baseUrl,
    model,
    messages: allMessages,
    tools: tools.value
  })
}

const handleToolCalls = async (toolCalls: any[]) => {
  const results: ChatMessage[] = []
  for (const toolCall of toolCalls) {
    const { name, arguments: argsString } = toolCall.function
    
    // Show tool call in UI
    messages.value.push({ 
      role: 'assistant', 
      content: `[正在调用工具: ${name}...]`,
      name: 'system_info' 
    })

    try {
      const result = await invoke<any>('execute_tool', { name, arguments: argsString })
      
      if (result.needs_confirmation) {
        // Stop the loop and wait for user confirmation
        pendingToolCall.value = { toolCall, result }
        showConfirmModal.value = true
        return 'WAITING_FOR_CONFIRMATION'
      }

      results.push({
        role: 'tool',
        tool_call_id: toolCall.id,
        content: JSON.stringify(result.data)
      })
    } catch (err: any) {
      results.push({
        role: 'tool',
        tool_call_id: toolCall.id,
        content: JSON.stringify({ error: err.toString() })
      })
    }
  }
  return results
}

const sendMessage = async () => {
  if (!userInput.value.trim() || loading.value) return
  
  if (settingsStore.provider === 'deepseek' && !settingsStore.apiKey) {
    message.warning('请先配置 API Key')
    return
  }

  if (settingsStore.provider === 'ollama' && !settingsStore.ollamaModel) {
    message.warning('请先在设置中选择 Ollama 模型')
    return
  }

  const question = userInput.value
  messages.value.push({ role: 'user', content: question })
  userInput.value = ''
  loading.value = true

  try {
    let loopCount = 0
    const maxLoops = 5
    
    while (loopCount < maxLoops) {
      const response = await chatWithAI(messages.value.filter(m => m.name !== 'system_info'))
      
      if (response.tool_calls && response.tool_calls.length > 0) {
        // Add the assistant message with tool calls to history
        messages.value.push({
          role: 'assistant',
          content: response.content || '',
          tool_calls: response.tool_calls
        })

        const toolResults = await handleToolCalls(response.tool_calls)
        
        if (toolResults === 'WAITING_FOR_CONFIRMATION') {
          return // Loop will continue after user confirms
        }

        if (Array.isArray(toolResults)) {
          messages.value.push(...toolResults)
        }
      } else {
        messages.value.push({
          role: 'assistant',
          content: response.content || ''
        })
        break
      }
      loopCount++
    }
  } catch (err: any) {
    console.error(err)
    message.error(`AI 调用失败: ${err}`)
    messages.value.push({ role: 'assistant', content: '抱歉，请求失败。请检查配置或网络。' })
  } finally {
    loading.value = false
  }
}

const confirmToolExecution = async () => {
  if (!pendingToolCall.value) return
  confirmLoading.value = true
  
  try {
    const { toolCall, result } = pendingToolCall.value
    const args = JSON.parse(toolCall.function.arguments)
    
    let toolContent = ''
    if (result.action === 'create_log') {
      const newLog = {
        id: Date.now().toString(),
        title: args.title,
        content: args.content,
        date: new Date().toLocaleDateString(),
        tags: args.tags || []
      }
      await logStore.addLog(newLog)
      toolContent = JSON.stringify({ success: true, message: '日志已成功创建' })
    } else if (result.action === 'update_log') {
      await logStore.updateLog(args.id, {
        title: args.title,
        content: args.content,
        tags: args.tags
      })
      toolContent = JSON.stringify({ success: true, message: '日志已成功更新' })
    }

    // Push the result and continue chat
    messages.value.push({
      role: 'tool',
      tool_call_id: toolCall.id,
      content: toolContent
    })

    showConfirmModal.value = false
    pendingToolCall.value = null
    
    // Continue the chat loop
    await continueChatAfterTool()
  } catch (err: any) {
    message.error(`执行失败: ${err}`)
  } finally {
    confirmLoading.value = false
  }
}

const cancelToolExecution = () => {
  if (!pendingToolCall.value) return
  const { toolCall } = pendingToolCall.value
  
  messages.value.push({
    role: 'tool',
    tool_call_id: toolCall.id,
    content: JSON.stringify({ success: false, message: '用户取消了操作' })
  })

  showConfirmModal.value = false
  pendingToolCall.value = null
  continueChatAfterTool()
}

const continueChatAfterTool = async () => {
  loading.value = true
  try {
    const response = await chatWithAI(messages.value.filter(m => m.name !== 'system_info'))
    messages.value.push({
      role: 'assistant',
      content: response.content || ''
    })
  } catch (err: any) {
    message.error(`AI 继续对话失败: ${err}`)
  } finally {
    loading.value = false
  }
}

const clearChat = () => {
  messages.value = [{ role: 'assistant', content: '你好！我是你的智能日志助手。我可以帮你查找日志、统计工作情况，甚至帮你写日志。' }]
}
</script>

<template>
  <div class="flex flex-col h-full p-4 relative">
    <!-- Header -->
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
              <n-form-item label="供应商">
                <n-tag :type="settingsStore.provider === 'deepseek' ? 'primary' : 'success'" size="small">
                  {{ settingsStore.provider === 'deepseek' ? 'DeepSeek' : 'Ollama' }}
                </n-tag>
                <n-button size="tiny" quaternary class="ml-2" @click="message.info('请在右上角设置中更改供应商')">更改</n-button>
              </n-form-item>
              <n-form-item label="模型">
                <div class="text-xs truncate max-w-[200px]">
                  {{ settingsStore.provider === 'deepseek' ? settingsStore.model : settingsStore.ollamaModel || '未选择' }}
                </div>
              </n-form-item>
            </n-form>
          </div>
        </n-popover>
      </div>
    </div>
    
    <!-- Chat Messages -->
    <n-scrollbar class="flex-1 pr-2">
      <div class="space-y-4 pb-4">
        <div
          v-for="(msg, index) in messages"
          :key="index"
          :class="['flex', msg.role === 'user' ? 'justify-end' : 'justify-start']"
        >
          <!-- Tool info message -->
          <div v-if="msg.name === 'system_info'" class="flex items-center space-x-1 text-xs text-gray-400 italic">
            <n-icon size="14"><BuildOutline /></n-icon>
            <span>{{ msg.content }}</span>
          </div>

          <!-- Normal message -->
          <div
            v-else-if="msg.role !== 'tool'"
            :class="[
              'max-w-[85%] p-3 rounded-lg text-sm whitespace-pre-wrap shadow-sm',
              msg.role === 'user' 
                ? 'bg-blue-600 text-white rounded-br-none' 
                : 'bg-gray-100 dark:bg-zinc-800 rounded-bl-none border border-gray-200 dark:border-zinc-700'
            ]"
          >
            {{ msg.content }}
          </div>
        </div>
      </div>
    </n-scrollbar>

    <!-- Input Area -->
    <div class="mt-4 pt-4 border-t border-gray-200 dark:border-zinc-800 bg-white dark:bg-zinc-900 sticky bottom-0">
      <n-input
        v-model:value="userInput"
        type="textarea"
        placeholder="输入指令 (如: 总结今天的工作)..."
        :autosize="{ minRows: 2, maxRows: 5 }"
        :disabled="loading"
        @keypress.enter.prevent="sendMessage"
      />
      <div class="flex justify-between items-center mt-2">
        <div class="text-xs text-gray-400">
          {{ loading ? 'AI 正在思考中...' : '支持工具调用' }}
        </div>
        <n-button type="primary" :loading="loading" @click="sendMessage" :disabled="!userInput.trim()">
          <template #icon>
            <SendOutline />
          </template>
          发送
        </n-button>
      </div>
    </div>

    <!-- Confirmation Modal -->
    <n-modal v-model:show="showConfirmModal" :mask-closable="false">
      <n-card
        style="width: 400px"
        title="确认操作"
        :bordered="false"
        size="huge"
        role="dialog"
        aria-modal="true"
      >
        <template #header-extra>
          <n-icon size="24" color="#f0a020"><BuildOutline /></n-icon>
        </template>
        
        <div v-if="pendingToolCall">
          <p class="mb-4">AI 想要执行以下操作：</p>
          <div class="bg-gray-50 dark:bg-zinc-800 p-3 rounded text-sm mb-4">
            <div class="font-bold text-blue-600 mb-1">
              {{ pendingToolCall.result.action === 'create_log' ? '新建日志' : '修改日志' }}
            </div>
            <div class="space-y-1">
              <div><span class="text-gray-400">标题:</span> {{ JSON.parse(pendingToolCall.toolCall.function.arguments).title }}</div>
              <div class="truncate"><span class="text-gray-400">内容:</span> {{ JSON.parse(pendingToolCall.toolCall.function.arguments).content }}</div>
            </div>
          </div>
          <p class="text-xs text-gray-500 italic">此操作将修改您的本地数据。</p>
        </div>

        <template #footer>
          <n-space justify="end">
            <n-button @click="cancelToolExecution" :disabled="confirmLoading">
              <template #icon><n-icon><CloseCircleOutline /></n-icon></template>
              取消
            </n-button>
            <n-button type="primary" @click="confirmToolExecution" :loading="confirmLoading">
              <template #icon><n-icon><CheckmarkCircleOutline /></n-icon></template>
              确认执行
            </n-button>
          </n-space>
        </template>
      </n-card>
    </n-modal>
  </div>
</template>

<style scoped>
/* Ensure content doesn't jump when info messages appear */
.v-enter-active,
.v-leave-active {
  transition: opacity 0.3s ease;
}

.v-enter-from,
.v-leave-to {
  opacity: 0;
}
</style>
