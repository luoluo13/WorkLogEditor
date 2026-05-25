<script setup lang="ts">
import { ref, watch, onMounted, onUnmounted, computed } from 'vue'
import { MdEditor, NormalToolbar } from 'md-editor-v3'
import 'md-editor-v3/lib/style.css'
import { useLogStore } from '../stores/log'
import { useSettingsStore } from '../stores/settings'
import { NInput, NButton, useNotification, NDropdown, NIcon, NDynamicTags, NColorPicker, NPopover } from 'naive-ui'
import { save } from '@tauri-apps/plugin-dialog'
import { invoke } from '@tauri-apps/api/core'
import { DownloadOutline, PricetagsOutline, AddOutline, ColorPaletteOutline } from '@vicons/ionicons5'

const logStore = useLogStore()
const settingsStore = useSettingsStore()
const notification = useNotification()

const content = ref('')
const title = ref('')
const tags = ref<string[]>([])
const selectedColor = ref('#6BCB77')
const coloredTextContent = ref('')
const editorRef = ref()
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

const toolbars: any[] = [
  'bold',
  'underline',
  'italic',
  '-',
  'strikeThrough',
  'title',
  'sub',
  'sup',
  'quote',
  'unorderedList',
  'orderedList',
  'task',
  '-',
  'codeRow',
  'code',
  'link',
  'image',
  'table',
  'mermaid',
  'katex',
  '-',
  'revoke',
  'next',
  'save',
  '=',
  0, // Custom color picker
  'preview',
  'htmlPreview',
  'catalog'
]

const insertColoredText = () => {
  if (!coloredTextContent.value.trim()) {
    notification.warning({ content: '请输入要插入的内容' })
    return
  }

  const coloredSnippet = `<span style="color: ${selectedColor.value}">${coloredTextContent.value}</span>`
  
  if (editorRef.value) {
    editorRef.value.insert(() => {
      return {
        targetValue: coloredSnippet,
        select: false,
        deviationStart: 0,
        deviationEnd: 0
      }
    })
    // 清空输入框以便下次使用
    coloredTextContent.value = ''
  } else {
    // 回退方案：如果 ref 没拿到，直接追加到末尾
    content.value += coloredSnippet
  }

  // 触发自动保存
  onAutoSave()
}

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
  <div v-if="logStore.currentLog" class="h-full flex flex-col space-y-4 p-4 bg-crayon-bg crayon-texture relative overflow-hidden">
    <!-- Decorative Doodles -->
    <div class="absolute right-10 bottom-20 w-32 h-32 text-crayon-yellow opacity-10 animate-float pointer-events-none">
      <svg viewBox="0 0 100 100" fill="currentColor">
        <path d="M50 20l10 25h25l-20 15 8 25-23-15-23 15 8-25-20-15h25z" />
      </svg>
    </div>

    <div class="flex flex-col md:flex-row md:items-center space-y-2 md:space-y-0 md:space-x-4 z-10">
      <n-input
        v-model:value="title"
        placeholder="输入标题..."
        class="!text-xl md:!text-2xl font-bold flex-1 !bg-white !border-2 !border-gray-800 rough-border shadow-sm"
        :bordered="false"
        @blur="onAutoSave"
      />
      <div class="flex justify-end space-x-2">
        <n-dropdown :options="exportOptions" @select="handleExport">
          <n-button quaternary circle class="!text-gray-600">
            <template #icon><n-icon><DownloadOutline /></n-icon></template>
          </n-button>
        </n-dropdown>
        <n-button 
          @click="onManualSave" 
          class="!bg-crayon-blue !text-white !border-none rough-border shadow-sm hover:shadow-md transition-all font-bold"
        >
          保存日志
        </n-button>
      </div>
    </div>

    <!-- Tags Editor -->
    <div class="flex items-center px-2 min-h-[34px] z-10 bg-white/50 rounded-full py-1 self-start rough-border border-gray-300">
      <n-icon :component="PricetagsOutline" class="mr-2 text-crayon-purple flex-shrink-0" size="18" />
      <n-dynamic-tags v-model:value="tags" @update:value="onAutoSave" class="custom-tags">
        <template #trigger="{ activate, disabled }">
          <n-button
            size="small"
            dashed
            :disabled="disabled"
            @click="activate()"
            class="!rounded-full"
          >
            <template #icon>
              <n-icon><AddOutline /></n-icon>
            </template>
            贴个标签
          </n-button>
        </template>
      </n-dynamic-tags>
    </div>
    
    <div class="flex-1 overflow-hidden rough-border border-gray-800 bg-white shadow-lg z-10">
      <MdEditor
        ref="editorRef"
        v-model="content"
        :theme="settingsStore.isDarkMode ? 'dark' : 'light'"
        :toolbars="toolbars"
        preview-theme="github"
        class="h-full custom-md-editor"
        @onSave="handleSave"
      >
        <template #defToolbars>
          <NormalToolbar title="插入彩色文字">
            <template #trigger>
              <n-popover trigger="click" placement="bottom" :width="240">
                <template #trigger>
                  <n-icon size="20" class="cursor-pointer hover:text-crayon-purple transition-colors">
                    <ColorPaletteOutline />
                  </n-icon>
                </template>
                <div class="p-3 space-y-3">
                  <p class="text-sm font-hand text-gray-500 flex items-center">
                    <span class="w-2 h-2 rounded-full bg-crayon-purple mr-2"></span>
                    选择画笔颜色与文字 ✨
                  </p>
                  
                  <n-color-picker 
                    v-model:value="selectedColor" 
                    :show-alpha="false" 
                    :modes="['hex']"
                  />

                  <n-input 
                    v-model:value="coloredTextContent" 
                    placeholder="在这里输入想要变色的文字..."
                    size="small"
                    class="rough-border !border-gray-200"
                    @keyup.enter="insertColoredText"
                  />

                  <n-button 
                    size="small" 
                    block 
                    class="!bg-crayon-purple !text-white !border-none !rounded-full shadow-sm hover:shadow-md transition-all font-bold" 
                    @click="insertColoredText"
                  >
                    确认插入
                  </n-button>
                </div>
              </n-popover>
            </template>
          </NormalToolbar>
        </template>
      </MdEditor>
    </div>

    <div class="flex justify-between items-center text-sm text-gray-400 px-2 pb-1 z-10 font-hand">
      <span class="flex items-center">
        <span class="w-2 h-2 rounded-full bg-crayon-green mr-2"></span>
        {{ logStore.currentLog.date }}
      </span>
      <span>共 {{ wordCount }} 个字</span>
    </div>
  </div>
  <div v-else class="h-full flex flex-col items-center justify-center bg-crayon-bg crayon-texture">
    <div class="w-48 h-48 text-crayon-blue opacity-30 animate-float mb-4">
      <svg viewBox="0 0 100 100" fill="currentColor">
        <path d="M20 50c0-10 10-15 20-15 5-10 20-10 25 0 10 0 15 10 15 15 0 10-10 15-20 15H35c-10 0-15-5-15-15z" />
      </svg>
    </div>
    <p class="text-xl text-gray-400 font-hand">选一个日志，开始今天的奇思妙想吧 ✨</p>
  </div>
</template>

<style>
.md-editor {
  height: 100% !important;
  border: none !important;
  background-color: transparent !important;
}

.custom-md-editor .md-editor-toolbar {
  border-bottom: 2px solid #f0f0f0 !important;
  background-color: #fafafa !important;
}

.custom-md-editor .md-editor-content {
  font-family: 'Patrick Hand', 'Ma Shan Zheng', cursive !important;
}

.custom-tags :deep(.n-tag) {
  background-color: #F3E8FF !important;
  color: #9333EA !important;
  border: 1px solid #E9D5FF !important;
  border-radius: 12px !important;
  font-family: 'Patrick Hand', 'Ma Shan Zheng', cursive !important;
}
</style>
