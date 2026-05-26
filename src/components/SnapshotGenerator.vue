<script setup lang="ts">
import { ref } from 'vue'
import { NModal, NCard, NButton, NSpace, NIcon, useMessage } from 'naive-ui'
import { DownloadOutline, CloseOutline } from '@vicons/ionicons5'
import { LogEntry } from '../stores/log'
import html2canvas from 'html2canvas'
import { invoke } from '@tauri-apps/api/core'
import { MdPreview } from 'md-editor-v3'
import 'md-editor-v3/lib/preview.css'

const props = defineProps<{
  show: boolean
  log: LogEntry | null
}>()

const emit = defineEmits(['update:show'])

const message = useMessage()
const snapshotRef = ref<HTMLElement | null>(null)
const isGenerating = ref(false)

const handleClose = () => {
  emit('update:show', false)
}

const downloadSnapshot = async () => {
  if (!snapshotRef.value || !props.log) return
  
  isGenerating.value = true
  try {
    const canvas = await html2canvas(snapshotRef.value, {
      backgroundColor: '#fdf6e3',
      scale: 2,
      useCORS: true,
      logging: false,
      allowTaint: true,
      onclone: (clonedDoc) => {
        const element = clonedDoc.querySelector('.hand-drawn-container') as HTMLElement
        if (element) {
          element.style.height = 'auto'
          element.style.maxHeight = 'none'
          element.style.overflow = 'visible'
        }
      }
    })
    
    const base64Data = canvas.toDataURL('image/png')
    const filename = `snapshot_${props.log.date.replace(/\//g, '-')}_${props.log.title.replace(/\s+/g, '_')}.png`
    
    await invoke('save_snapshot', { base64Data, filename })
    message.success('快照已保存')
    handleClose()
  } catch (err: any) {
    console.error('Failed to generate snapshot:', err)
    message.error(`生成快照失败: ${err}`)
  } finally {
    isGenerating.value = false
  }
}
</script>

<template>
  <n-modal :show="show" @update:show="emit('update:show', $event)">
    <n-card
      style="width: 700px; max-width: 95vw"
      title="日志快照预览"
      :bordered="false"
      size="huge"
      role="dialog"
      aria-modal="true"
    >
      <template #header-extra>
        <n-button quaternary circle @click="handleClose">
          <template #icon><n-icon><CloseOutline /></n-icon></template>
        </n-button>
      </template>

      <div class="snapshot-preview-area p-8 bg-gray-200 rounded-lg overflow-auto max-h-[75vh]">
        <div class="flex justify-center w-full">
          <div ref="snapshotRef" class="hand-drawn-container bg-[#fdf6e3] shadow-2xl relative min-w-[600px] max-w-[800px] flex flex-col h-auto">
            <!-- Rough paper effect -->
            <div class="paper-texture absolute inset-0 opacity-15 pointer-events-none z-0"></div>
            
            <div class="content-wrapper relative z-10 p-12 h-auto">
              <h1 class="text-4xl font-bold mb-6 border-b-2 border-gray-800 pb-4 font-hand">{{ log?.title }}</h1>
              
              <div class="flex justify-between items-center text-sm mb-8 text-gray-700 font-hand italic">
                <span class="bg-gray-800/5 px-3 py-1 rounded">📅 {{ log?.date }}</span>
                <div class="flex gap-2">
                  <span v-for="tag in log?.tags" :key="tag" class="px-2 py-1 border border-gray-800/30 rounded-md bg-white/50">
                    #{{ tag }}
                  </span>
                </div>
              </div>

              <div class="log-markdown-content h-auto">
                <MdPreview 
                  :modelValue="log?.content || ''" 
                  preview-theme="github"
                  class="hand-drawn-md-preview"
                />
              </div>
              
              <div class="mt-16 pt-6 border-t-2 border-dashed border-gray-800/30 flex justify-between items-center opacity-50 text-xs font-hand">
                <div class="flex items-center gap-2">
                  <div class="w-4 h-4 text-gray-800">
                    <svg viewBox="0 0 100 100" fill="currentColor">
                      <path d="M50 80c-10 0-25-10-25-25 0-15 15-20 25-10 10-10 25-5 25 10 0 15-15 25-25 25z" />
                    </svg>
                  </div>
                  <span>由 AI 智能日志编辑器生成</span>
                </div>
                <span>© {{ new Date().getFullYear() }}</span>
              </div>
            </div>

            <!-- Decorative elements for hand-drawn look -->
            <div class="absolute -top-2 -left-2 w-8 h-8 border-t-4 border-l-4 border-gray-800/20 rounded-tl-xl pointer-events-none"></div>
            <div class="absolute -bottom-2 -right-2 w-8 h-8 border-b-4 border-r-4 border-gray-800/20 rounded-br-xl pointer-events-none"></div>
          </div>
        </div>
      </div>

      <template #footer>
        <n-space justify="end">
          <n-button @click="handleClose">取消</n-button>
          <n-button type="primary" :loading="isGenerating" @click="downloadSnapshot">
            <template #icon><n-icon><DownloadOutline /></n-icon></template>
            保存并导出
          </n-button>
        </n-space>
      </template>
    </n-card>
  </n-modal>
</template>

<style scoped>
@import url('https://fonts.googleapis.com/css2?family=Kalam:wght@300;400;700&display=swap');

.font-hand {
  font-family: 'Kalam', cursive;
}

.hand-drawn-container {
  /* Irregular hand-drawn border effect */
  border: 3px solid #2c3e50;
  border-radius: 2% 6% 5% 4% / 1% 1% 2% 4%;
  box-shadow: 15px 15px 30px rgba(0, 0, 0, 0.1);
  min-height: 400px;
}

.paper-texture {
  background-image: url("data:image/svg+xml,%3Csvg viewBox='0 0 200 200' xmlns='http://www.w3.org/2000/svg'%3E%3Cfilter id='noiseFilter'%3E%3CfeTurbulence type='fractalNoise' baseFrequency='0.65' numOctaves='3' stitchTiles='stitch'/%3E%3C/filter%3E%3Crect width='100%25' height='100%25' filter='url(%23noiseFilter)'/%3E%3C/svg%3E");
  z-index: 0;
}

/* Override MdPreview styles for hand-drawn look */
:deep(.hand-drawn-md-preview) {
  background-color: transparent !important;
  font-family: 'Kalam', cursive !important;
  font-size: 1.25rem !important;
  line-height: 1.8 !important;
  color: #2c3e50 !important;
}

:deep(.hand-drawn-md-preview .md-editor-preview) {
  padding: 0 !important;
}

:deep(.hand-drawn-md-preview h1, 
      .hand-drawn-md-preview h2, 
      .hand-drawn-md-preview h3, 
      .hand-drawn-md-preview h4) {
  border-bottom: 2px solid rgba(44, 62, 80, 0.2) !important;
  margin-top: 1.5rem !important;
  margin-bottom: 1rem !important;
  color: #1a202c !important;
  font-family: 'Kalam', cursive !important;
}

:deep(.hand-drawn-md-preview blockquote) {
  border-left: 4px solid #6BCB77 !important;
  background-color: rgba(107, 203, 119, 0.05) !important;
  padding: 1rem !important;
  border-radius: 4px !important;
  margin: 1.5rem 0 !important;
  font-style: italic !important;
}

:deep(.hand-drawn-md-preview ul, .hand-drawn-md-preview ol) {
  padding-left: 2rem !important;
  margin: 1rem 0 !important;
}

:deep(.hand-drawn-md-preview code) {
  background-color: rgba(0, 0, 0, 0.05) !important;
  padding: 0.2rem 0.4rem !important;
  border-radius: 4px !important;
  font-family: monospace !important;
  font-size: 0.9em !important;
}

:deep(.hand-drawn-md-preview pre code) {
  display: block !important;
  padding: 1rem !important;
  overflow-x: auto !important;
  background-color: #282c34 !important;
  color: #abb2bf !important;
  border-radius: 8px !important;
  box-shadow: 4px 4px 10px rgba(0, 0, 0, 0.1) !important;
}

/* Add a slight rotation to some elements to make it look more hand-drawn */
h1 {
  transform: rotate(-0.5deg);
}

.content-wrapper {
  position: relative;
  width: 100%;
}
</style>
