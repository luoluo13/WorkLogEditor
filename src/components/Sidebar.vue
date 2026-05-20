<script setup lang="ts">
import { ref, watch, computed } from 'vue'
import { NList, NListItem, NScrollbar, NButton, NIcon, NInput, NTag, NSpace, NDropdown, useNotification, NCalendar, NTabs, NTabPane, NPopconfirm } from 'naive-ui'
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
  <div class="flex flex-col h-full">
    <div class="p-4 pb-0">
      <div class="flex space-x-2 mb-4">
        <n-button type="primary" block @click="createNewLog" class="flex-1">
          <template #icon>
            <n-icon><AddOutline /></n-icon>
          </template>
          新建日志
        </n-button>
        <n-dropdown :options="moreOptions" @select="handleMoreAction">
          <n-button quaternary circle>
            <template #icon><n-icon><EllipsisVerticalOutline /></n-icon></template>
          </n-button>
        </n-dropdown>
      </div>
      <n-input v-model:value="searchKeyword" placeholder="搜索日志..." class="mb-4">
        <template #prefix>
          <n-icon><SearchOutline /></n-icon>
        </template>
      </n-input>
      
      <n-tabs v-model:value="currentTab" type="segment" size="small">
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
    
    <n-scrollbar class="flex-1">
      <div v-if="currentTab === 'list'">
        <n-list hoverable clickable>
          <n-list-item
            v-for="log in logStore.logs"
            :key="log.id"
            @click="logStore.currentLog = log"
            :class="{ 'bg-blue-50 dark:bg-blue-900/20': logStore.currentLog?.id === log.id }"
            class="relative group"
          >
            <div class="flex flex-col space-y-1 pr-8">
              <span class="font-medium truncate">{{ log.title }}</span>
              <div class="flex justify-between items-center">
                <span class="text-xs text-gray-500">{{ log.date }}</span>
                <n-space :size="4" v-if="log.tags && log.tags.length > 0">
                  <n-tag v-for="tag in log.tags.slice(0, 2)" :key="tag" size="tiny" :bordered="false">
                    {{ tag }}
                  </n-tag>
                </n-space>
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
                确定要删除这条日志吗？
              </n-popconfirm>
            </div>
          </n-list-item>
        </n-list>
      </div>
      <div v-else class="p-2 calendar-container">
        <n-calendar
          #default="{ year, month, date }"
          @update:value="handleDateClick"
        >
          <div v-if="logDates.has(`${year}/${month}/${date}`)" class="flex justify-center mt-1">
            <div class="w-1.5 h-1.5 rounded-full bg-blue-500"></div>
          </div>
        </n-calendar>
      </div>
    </n-scrollbar>
  </div>
</template>

<style scoped>
.calendar-container :deep(.n-calendar) {
  --n-cell-height: 40px !important;
  font-size: 12px;
}

.calendar-container :deep(.n-calendar-header) {
  padding: 8px;
  font-size: 14px;
}

@media (max-width: 768px) {
  .calendar-container :deep(.n-calendar) {
    --n-cell-height: 50px !important;
  }
}
</style>
