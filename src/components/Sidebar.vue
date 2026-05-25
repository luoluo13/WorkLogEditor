<script setup lang="ts">
import { ref, watch, computed } from 'vue'
import { NScrollbar, NButton, NIcon, NInput, NDropdown, useNotification, NCalendar, NTabs, NTabPane, NPopconfirm } from 'naive-ui'
import { AddOutline, SearchOutline, EllipsisVerticalOutline, CalendarOutline, ListOutline, TrashOutline } from '@vicons/ionicons5'
import { save, open } from '@tauri-apps/plugin-dialog'
import { invoke } from '@tauri-apps/api/core'
import { useLogStore } from '../stores/log'

const logStore = useLogStore()
const notification = useNotification()

const currentTab = ref('list')
const searchKeyword = ref('')
let searchTimeout: number | null = null

const logDates = computed(() => {
  return new Set(logStore.logs.map(log => log.date))
})

const handleDateClick = async (date: number) => {
  const dateStr = new Date(date).toLocaleDateString()
  const log = logStore.logs.find(l => l.date === dateStr)
  if (log) {
    logStore.currentLog = log
    currentTab.value = 'list'
  }
}

const moreOptions = [
  { label: '批量导出 (ZIP)', key: 'export_zip' },
  { label: '导入 Markdown', key: 'import_md' }
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
      try {
        const importedLog = await invoke<any>('import_from_markdown', { path: filePath })
        await logStore.addLog(importedLog)
        notification.success({ content: '导入成功' })
      } catch (err) {
        notification.error({ content: '导入失败' })
      }
    }
  }
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
        <div class="space-y-3">
          <div
            v-for="log in logStore.logs"
            :key="log.id"
            @click="logStore.currentLog = log"
            :class="[
              'p-3 cursor-pointer transition-all rough-border bg-white hover:translate-x-1',
              logStore.currentLog?.id === log.id ? 'border-crayon-blue !border-4 ring-2 ring-crayon-blue/20 shadow-inner' : 'border-gray-300'
            ]"
            class="relative group"
          >
            <div class="flex flex-col space-y-1 pr-8">
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
        </div>
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
