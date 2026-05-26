<script setup lang="ts">
import { ref, watch, computed } from 'vue'
import { NScrollbar, NButton, NIcon, NInput, NDropdown, useNotification, NCalendar, NTabs, NTabPane, NPopconfirm, NDatePicker, NRadioGroup, NRadioButton, NSpace, NModal } from 'naive-ui'
import { AddOutline, SearchOutline, EllipsisVerticalOutline, CalendarOutline, ListOutline, TrashOutline, SwapVerticalOutline } from '@vicons/ionicons5'
import { save, open } from '@tauri-apps/plugin-dialog'
import { invoke } from '@tauri-apps/api/core'
import { useLogStore } from '../stores/log'
import draggable from 'vuedraggable'

const logStore = useLogStore()
const notification = useNotification()

const currentTab = ref('list')
const searchKeyword = ref('')
const importDate = ref<number | null>(Date.now())
const showImportDatePicker = ref(false)
const pendingImportPath = ref<string | null>(null)
let searchTimeout: number | null = null

const displayLogs = computed({
  get: () => logStore.filteredLogs,
  set: (val) => logStore.reorderLogs(val)
})

const logDates = computed(() => {
  return new Set(logStore.logs.map((log: any) => log.date))
})

const handleDateClick = async (date: number) => {
  const dateStr = new Date(date).toLocaleDateString()
  logStore.filterDate = dateStr
  currentTab.value = 'list'
}

const moreOptions = [
  { label: '批量导出 (ZIP)', key: 'export_zip' },
  { label: '导入 Markdown', key: 'import_md' },
  { type: 'divider', key: 'd1' },
  { label: '按日期降序', key: 'sort_desc' },
  { label: '按日期升序', key: 'sort_asc' }
]

const handleMoreAction = async (key: string) => {
  if (key === 'export_zip') {
    if (logStore.logs.length === 0) {
      notification.warning({ content: '没有可导出的日志' })
      return
    }
    const filePath = await save({
      filters: [{ name: 'ZIP Archive', extensions: ['zip'] }],
      defaultPath: 'worklogs_backup.zip'
    })
    if (filePath) {
      try {
        await invoke('export_logs_as_zip', { logs: logStore.logs, zipPath: filePath })
        notification.success({ content: '批量导出成功' })
      } catch (err) {
        notification.error({ content: '导出失败' })
      }
    }
  } else if (key === 'import_md') {
    const filePath = await open({
      filters: [{ name: 'Markdown', extensions: ['md'] }],
      multiple: false
    })
    
    if (filePath && typeof filePath === 'string') {
      pendingImportPath.value = filePath
      showImportDatePicker.value = true
    }
  } else if (key === 'sort_desc') {
    logStore.sortLogsByDate('desc')
  } else if (key === 'sort_asc') {
    logStore.sortLogsByDate('asc')
  }
}

const confirmImport = async () => {
  if (!pendingImportPath.value) return
  
  try {
    const dateStr = importDate.value ? new Date(importDate.value).toLocaleDateString() : new Date().toLocaleDateString()
    const importedLog = await invoke<any>('import_from_markdown', { 
      path: pendingImportPath.value,
      date: dateStr
    })
    await logStore.addLog(importedLog)
    notification.success({ content: `已导入至 ${dateStr}` })
    showImportDatePicker.value = false
    pendingImportPath.value = null
  } catch (err) {
    notification.error({ content: '导入失败' })
  }
}

const cancelImport = () => {
  showImportDatePicker.value = false
  pendingImportPath.value = null
}

watch(searchKeyword, (newVal) => {
  if (searchTimeout) clearTimeout(searchTimeout)
  searchTimeout = window.setTimeout(() => {
    logStore.searchLogs(newVal)
  }, 300)
})

const createNewLog = async () => {
  const newLog = {
    id: Date.now().toString(),
    title: '未命名日志',
    content: '',
    date: new Date().toLocaleDateString(),
    tags: []
  }
  await logStore.addLog(newLog)
}

const deleteLog = async (id: string) => {
  try {
    await logStore.deleteLog(id)
    notification.success({ content: '日志已删除' })
  } catch (err) {
    notification.error({ content: '删除失败' })
  }
}
</script>

<template>
  <div class="flex flex-col h-full bg-crayon-bg crayon-texture relative overflow-hidden">
    <!-- Decorative Doodles -->
    <div class="absolute -left-4 -bottom-4 w-24 h-24 text-crayon-pink opacity-20 animate-float pointer-events-none">
      <svg viewBox="0 0 100 100" fill="currentColor">
        <path d="M50 80c-10 0-25-10-25-25 0-15 15-20 25-10 10-10 25-5 25 10 0 15-15 25-25 25z" />
      </svg>
    </div>
    <div class="absolute -right-6 top-20 w-16 h-16 text-crayon-blue opacity-20 animate-float pointer-events-none" style="animation-delay: 1s;">
      <svg viewBox="0 0 100 100" fill="currentColor">
        <path d="M20 50c0-10 10-15 20-15 5-10 20-10 25 0 10 0 15 10 15 15 0 10-10 15-20 15H35c-10 0-15-5-15-15z" />
      </svg>
    </div>

    <div class="p-4 pb-0 z-10">
      <div class="flex space-x-2 mb-4">
        <n-button 
          block 
          @click="createNewLog" 
          class="flex-1 !bg-crayon-green !text-white !border-none rough-border shadow-sm hover:shadow-md transition-shadow"
          style="height: 44px; font-weight: bold; font-size: 1.1rem;"
        >
          <template #icon>
            <n-icon><AddOutline /></n-icon>
          </template>
          新建日志
        </n-button>
        <n-dropdown :options="moreOptions" @select="handleMoreAction">
          <n-button quaternary circle class="!text-gray-600">
            <template #icon><n-icon><EllipsisVerticalOutline /></n-icon></template>
          </n-button>
        </n-dropdown>
      </div>

      <div class="relative mb-4">
        <n-input 
          v-model:value="searchKeyword" 
          placeholder="搜索日志..." 
          class="!bg-white !border-2 !border-gray-800 rough-border"
          :bordered="false"
        >
          <template #prefix>
            <n-icon class="text-gray-400"><SearchOutline /></n-icon>
          </template>
        </n-input>
      </div>

      <div class="mb-4">
        <n-radio-group v-model:value="logStore.filterDate" size="small" class="w-full">
          <n-radio-button value="all" class="flex-1 text-center">全部</n-radio-button>
          <n-radio-button value="today" class="flex-1 text-center">今日</n-radio-button>
        </n-radio-group>
      </div>
      
      <n-tabs v-model:value="currentTab" type="segment" size="small" class="custom-tabs">
        <n-tab-pane name="list" tab="列表">
          <template #tab>
            <div class="flex items-center space-x-1">
              <n-icon><ListOutline /></n-icon>
              <span>列表</span>
            </div>
          </template>
        </n-tab-pane>
        <n-tab-pane name="calendar" tab="日历">
          <template #tab>
            <div class="flex items-center space-x-1">
              <n-icon><CalendarOutline /></n-icon>
              <span>日历</span>
            </div>
          </template>
        </n-tab-pane>
      </n-tabs>
    </div>
    
    <n-scrollbar class="flex-1 z-10">
      <div v-if="currentTab === 'list'" class="p-4 pt-2">
        <draggable 
          v-model="displayLogs" 
          item-key="id" 
          handle=".drag-handle"
          class="space-y-3"
          :animation="200"
        >
          <template #item="{ element: log }">
            <div
              @click="logStore.currentLog = log"
              :class="[
                'p-3 cursor-pointer transition-all rough-border bg-white hover:translate-x-1',
                logStore.currentLog?.id === log.id ? 'border-crayon-blue !border-4 ring-2 ring-crayon-blue/20 shadow-inner' : 'border-gray-300'
              ]"
              class="relative group"
            >
              <div class="flex items-start">
                <!-- Drag Handle -->
                <div class="drag-handle mr-2 mt-1 cursor-move opacity-30 group-hover:opacity-100 transition-opacity">
                  <n-icon><SwapVerticalOutline /></n-icon>
                </div>
                
                <div class="flex flex-col space-y-1 pr-8 flex-1">
                  <span class="font-bold text-gray-700 truncate text-base">{{ log.title }}</span>
                  <div class="flex justify-between items-center">
                    <span class="text-xs text-gray-400 italic">{{ log.date }}</span>
                    <div class="flex space-x-1" v-if="log.tags && log.tags.length > 0">
                      <span 
                        v-for="tag in log.tags.slice(0, 2)" 
                        :key="tag" 
                        class="px-2 py-0.5 text-[10px] bg-crayon-purple/20 text-crayon-purple rounded-full border border-crayon-purple/30"
                      >
                        #{{ tag }}
                      </span>
                    </div>
                  </div>
                </div>
              </div>

              <!-- Delete Button -->
              <div class="absolute right-2 top-1/2 -translate-y-1/2 opacity-0 group-hover:opacity-100 transition-opacity">
                <n-popconfirm @positive-click.stop="deleteLog(log.id)" @click.stop>
                  <template #trigger>
                    <n-button quaternary circle size="small" type="error">
                      <template #icon>
                        <n-icon><TrashOutline /></n-icon>
                      </template>
                    </n-button>
                  </template>
                  确定删除吗？
                </n-popconfirm>
              </div>
            </div>
          </template>
        </draggable>
      </div>
      <div v-else class="p-4 calendar-container">
        <div class="bg-white p-2 rough-border">
          <n-calendar
            #default="{ year, month, date }"
            @update:value="handleDateClick"
          >
            <div v-if="logDates.has(`${year}/${month}/${date}`)" class="flex justify-center mt-1">
              <div class="w-2 h-2 rounded-full bg-crayon-pink animate-pulse"></div>
            </div>
          </n-calendar>
        </div>
      </div>
    </n-scrollbar>

    <!-- Import Date Modal -->
    <n-modal v-model:show="showImportDatePicker" preset="card" title="选择导入日期" style="width: 400px">
      <div class="p-4 space-y-4">
        <p class="text-gray-500">请选择要将 Markdown 导入到哪一天的日志：</p>
        <n-date-picker v-model:value="importDate" type="date" class="w-full" />
        <n-space justify="end">
          <n-button @click="cancelImport">取消</n-button>
          <n-button type="primary" @click="confirmImport">确认导入</n-button>
        </n-space>
      </div>
    </n-modal>
  </div>
</template>

<style scoped>
.calendar-container :deep(.n-calendar) {
  --n-cell-height: 40px !important;
  font-family: 'Patrick Hand', 'Ma Shan Zheng', cursive !important;
}

.calendar-container :deep(.n-calendar-header) {
  padding: 8px;
}

.custom-tabs :deep(.n-tabs-rail) {
  background-color: #eee !important;
  border-radius: 20px !important;
  padding: 2px !important;
}

.custom-tabs :deep(.n-tabs-tab--active) {
  background-color: white !important;
  box-shadow: 0 2px 4px rgba(0,0,0,0.1) !important;
  border-radius: 18px !important;
}
</style>
