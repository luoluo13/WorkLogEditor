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
      // Switch to the newly created log
      logStore.currentLog = newLog
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
  <div class="flex flex-col h-full bg-crayon-bg crayon-texture relative overflow-hidden">
    <!-- Decorative Doodles -->
    <div class="absolute -right-2 top-4 w-16 h-16 text-crayon-purple opacity-20 animate-float pointer-events-none">
      <svg viewBox="0 0 100 100" fill="currentColor">
        <path d="M50 20l5 15h15l-12 10 5 15-13-10-13 10 5-15-12-10h15z" />
      </svg>
    </div>

    <div class="p-4 border-b border-gray-200 z-10 bg-white/50 backdrop-blur-sm">
      <div class="flex items-center justify-between">
        <div class="flex items-center space-x-2">
          <div class="w-8 h-8 rounded-full bg-crayon-purple flex items-center justify-center text-white rough-border !border-crayon-purple">
            <n-icon size="20"><ChatbubbleEllipsesOutline /></n-icon>
          </div>
          <span class="font-bold text-gray-700 font-hand text-lg">AI 助手</span>
        </div>
        <n-button quaternary circle size="small" @click="clearChat">
          <template #icon><n-icon><TrashOutline /></n-icon></template>
        </n-button>
      </div>
    </div>

    <n-scrollbar class="flex-1 p-4 z-10" ref="scrollbar">
      <div class="space-y-6">
        <div
          v-for="(msg, index) in messages"
          :key="index"
          :class="['flex', msg.role === 'user' ? 'justify-end' : 'justify-start']"
        >
          <div v-if="msg.name === 'system_info'" class="w-full text-center">
            <span class="text-[10px] bg-gray-100 text-gray-400 px-2 py-0.5 rounded-full italic font-hand">
              {{ msg.content }}
            </span>
          </div>
          <div
            v-else-if="msg.role !== 'tool'"
            :class="[
              'max-w-[85%] p-3 rough-border shadow-sm relative',
              msg.role === 'user' 
                ? 'bg-crayon-green text-white border-crayon-green !rounded-tr-none' 
                : 'bg-white text-gray-700 border-gray-300 !rounded-tl-none'
            ]"
          >
            <!-- Message Tail -->
            <div 
              :class="[
                'absolute top-0 w-4 h-4',
                msg.role === 'user' ? '-right-2 text-crayon-green' : '-left-2 text-white'
              ]"
            >
              <svg viewBox="0 0 20 20" fill="currentColor">
                <path d="M0 0 L20 0 L10 10 Z" v-if="msg.role === 'assistant'" />
                <path d="M20 0 L0 0 L10 10 Z" v-else />
              </svg>
            </div>

            <div class="text-sm font-hand leading-relaxed whitespace-pre-wrap">
              {{ msg.content }}
            </div>
            <div v-if="msg.tool_calls" class="mt-2 pt-2 border-t border-dashed border-gray-200">
              <div v-for="call in msg.tool_calls" :key="call.id" class="text-[10px] text-gray-400 italic">
                正在执行: {{ call.function.name }}...
              </div>
            </div>
          </div>
        </div>
        <div v-if="loading" class="flex justify-start">
          <div class="bg-white p-3 rough-border border-gray-300 !rounded-tl-none animate-pulse">
            <div class="flex space-x-1">
              <div class="w-2 h-2 bg-gray-300 rounded-full"></div>
              <div class="w-2 h-2 bg-gray-300 rounded-full"></div>
              <div class="w-2 h-2 bg-gray-300 rounded-full"></div>
            </div>
          </div>
        </div>
      </div>
    </n-scrollbar>

    <div class="p-4 z-10 bg-white/50 backdrop-blur-sm border-t border-gray-200">
      <div class="relative">
        <n-input
          v-model:value="userInput"
          type="textarea"
          :autosize="{ minRows: 1, maxRows: 4 }"
          placeholder="输入指令 (如: 总结今天的工作)..."
          @keypress.enter.prevent="sendMessage"
          class="!bg-white !border-2 !border-gray-800 rough-border pr-12"
          :bordered="false"
        />
        <n-button
          type="primary"
          circle
          class="absolute right-2 bottom-2 !bg-crayon-green !border-none shadow-md"
          :loading="loading"
          @click="sendMessage"
        >
          <template #icon><n-icon><SendOutline /></n-icon></template>
        </n-button>
      </div>
    </div>

    <!-- Tool Confirmation Modal -->
    <n-modal v-model:show="showConfirmModal" :mask-closable="false">
      <n-card
        style="width: 400px"
        title="确认 AI 的建议"
        :bordered="false"
        size="huge"
        role="dialog"
        aria-modal="true"
        class="rough-border border-2 border-gray-800"
      >
        <template #header-extra>
          <n-icon size="24" class="text-crayon-yellow"><BuildOutline /></n-icon>
        </template>
        
        <div v-if="pendingToolCall" class="font-hand">
          <div class="mb-4 p-3 bg-crayon-yellow/20 rounded-lg border border-crayon-yellow/30">
            <div class="font-bold text-gray-700 mb-2">
              AI 想要执行以下操作：
            </div>
            <div class="text-sm text-gray-600">
               {{ pendingToolCall.result.action === 'create_log' ? '新建日志' : '修改日志' }}: {{ JSON.parse(pendingToolCall.toolCall.function.arguments).title }}
            </div>
          </div>
          
          <div class="flex justify-end space-x-3">
            <n-button @click="cancelToolExecution" quaternary class="!rounded-full" :disabled="confirmLoading">
              <template #icon><n-icon><CloseCircleOutline /></n-icon></template>
              取消
            </n-button>
            <n-button 
              @click="confirmToolExecution" 
              class="!bg-crayon-green !text-white !border-none !rounded-full shadow-sm"
              :loading="confirmLoading"
            >
              <template #icon><n-icon><CheckmarkCircleOutline /></n-icon></template>
              执行
            </n-button>
          </div>
        </div>
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
