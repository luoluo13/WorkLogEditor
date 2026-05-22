<script setup lang="ts">
import { ref, watch, onMounted, onUnmounted, computed } from 'vue'
import { MdEditor } from 'md-editor-v3'
import 'md-editor-v3/lib/style.css'
import { useLogStore } from '../stores/log'
import { useSettingsStore } from '../stores/settings'
import { NInput, NButton, useNotification, NDropdown, NIcon, NDynamicTags } from 'naive-ui'
import { save } from '@tauri-apps/plugin-dialog'
import { invoke } from '@tauri-apps/api/core'
import { DownloadOutline, PricetagsOutline, AddOutline } from '@vicons/ionicons5'

const logStore = useLogStore()
const settingsStore = useSettingsStore()
const notification = useNotification()

const content = ref('')
const title = ref('')
const tags = ref<string[]>([])
const wordCount = computed(() => {
  const text = content.value.trim()
  if (!text) return 0
  // Mixed English and Chinese word count
  const chineseChars = text.match(/[\u4e00-\u9fa5]/g) || []
  const englishWords = text.replace(/[\u4e00-\u9fa5]/g, ' ').split(/\s+/).filter(w => w.length > 0)
  return chineseChars.length + englishWords.length
})
let saveTimeout: number | null = null

const exportOptions = [
  { label: '导出为 Markdown', key: 'markdown' }
]

const handleExport = async (key: string) => {
  if (key === 'markdown' && logStore.currentLog) {
    const filePath = await save({
      filters: [{ name: 'Markdown', extensions: ['md'] }],
      defaultPath: `${logStore.currentLog.title}.md`
    })
    
    if (filePath) {
      try {
        await invoke('export_log_as_markdown', { 
          log: { ...logStore.currentLog, content: content.value, title: title.value },
          path: filePath 
        })
        notification.success({ content: '导出成功' })
      } catch (err) {
        notification.error({ content: '导出失败' })
      }
    }
  }
}

watch(() => logStore.currentLog, (newLog) => {
  if (newLog) {
    content.value = newLog.content
    title.value = newLog.title
    tags.value = [...(newLog.tags || [])]
  }
}, { immediate: true })

const handleSave = async (options?: { showNotification?: boolean }) => {
  if (logStore.currentLog) {
    await logStore.updateLog(logStore.currentLog.id, {
      content: content.value,
      title: title.value,
      tags: [...tags.value]
    })
    if (options?.showNotification !== false) {
      notification.success({
        content: '已保存',
        duration: 2000
      })
    }
  }
}

const onManualSave = () => handleSave({ showNotification: true })
const onAutoSave = () => handleSave({ showNotification: false })

// Auto-save logic
watch([content, title, tags], () => {
  if (saveTimeout) clearTimeout(saveTimeout)
  saveTimeout = window.setTimeout(() => {
    onAutoSave()
  }, 2000)
}, { deep: true })

// Shortcut logic
const handleKeydown = (e: KeyboardEvent) => {
  if ((e.ctrlKey || e.metaKey) && e.key === 's') {
    e.preventDefault()
    onManualSave()
  }
}

onMounted(() => {
  window.addEventListener('keydown', handleKeydown)
})

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeydown)
  if (saveTimeout) clearTimeout(saveTimeout)
})
</script>

<template>
  <div v-if="logStore.currentLog" class="h-full flex flex-col space-y-2 md:space-y-4">
    <div class="flex flex-col md:flex-row md:items-center space-y-2 md:space-y-0 md:space-x-4">
      <n-input
        v-model:value="title"
        placeholder="输入标题..."
        class="text-lg md:text-xl font-bold flex-1"
        @blur="onAutoSave"
      />
      <div class="flex justify-end space-x-2">
        <n-dropdown :options="exportOptions" @select="handleExport">
          <n-button quaternary circle>
            <template #icon><n-icon><DownloadOutline /></n-icon></template>
          </n-button>
        </n-dropdown>
        <n-button type="primary" @click="onManualSave">保存</n-button>
      </div>
    </div>

    <!-- Tags Editor -->
    <div class="flex items-center px-1 min-h-[34px]">
      <n-icon :component="PricetagsOutline" class="mr-2 text-gray-400 flex-shrink-0" size="18" />
      <n-dynamic-tags v-model:value="tags" @update:value="onAutoSave">
        <template #trigger="{ activate, disabled }">
          <n-button
            size="small"
            dashed
            :disabled="disabled"
            @click="activate()"
          >
            <template #icon>
              <n-icon><AddOutline /></n-icon>
            </template>
            添加标签
          </n-button>
        </template>
      </n-dynamic-tags>
    </div>
    
    <div class="flex-1 overflow-hidden rounded-lg border border-gray-200 dark:border-zinc-800">
      <MdEditor
        v-model="content"
        :theme="settingsStore.isDarkMode ? 'dark' : 'light'"
        :toolbars-exclude="['github']"
        preview-theme="github"
        class="h-full"
        @onSave="handleSave"
      />
    </div>

    <div class="flex justify-between items-center text-xs text-gray-500 px-1 pb-2">
      <span>{{ logStore.currentLog.date }}</span>
      <span>字数: {{ wordCount }}</span>
    </div>
  </div>
  <div v-else class="h-full flex items-center justify-center text-gray-500">
    选择或创建一个日志开始工作
  </div>
</template>

<style>
.md-editor {
  height: 100% !important;
}
</style>
