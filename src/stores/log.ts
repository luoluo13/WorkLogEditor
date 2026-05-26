import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export interface LogEntry {
  id: string
  title: string
  content: string
  date: string
  tags: string[]
}

export const useLogStore = defineStore('log', () => {
  const logs = ref<LogEntry[]>([])
  const currentLog = ref<LogEntry | null>(null)
  const isLoading = ref(false)
  const showSnapshot = ref(false)
  const filterDate = ref<string>('all') // 'all' means show all, 'today' means today, or YYYY/MM/DD

  const filteredLogs = computed(() => {
    let result = [...logs.value]
    
    // Date Filtering
    if (filterDate.value === 'today') {
      const today = new Date().toLocaleDateString()
      result = result.filter(l => l.date === today)
    } else if (filterDate.value && filterDate.value !== 'all') {
      result = result.filter(l => l.date === filterDate.value)
    }

    return result
  })

  async function fetchLogs() {
    isLoading.value = true
    try {
      logs.value = await invoke<LogEntry[]>('list_logs')
      if (logs.value.length > 0 && !currentLog.value) {
        currentLog.value = logs.value[0]
      }
    } catch (error) {
      console.error('Failed to fetch logs:', error)
    } finally {
      isLoading.value = false
    }
  }

  async function addLog(log: LogEntry) {
    try {
      await invoke('create_log', { log })
      logs.value.unshift(log)
      currentLog.value = log
    } catch (error) {
      console.error('Failed to add log:', error)
    }
  }

  async function updateLog(id: string, updates: Partial<LogEntry>) {
    const index = logs.value.findIndex(l => l.id === id)
    if (index !== -1) {
      const updatedLog = { ...logs.value[index], ...updates }
      try {
        await invoke('update_log', {
          id: updatedLog.id,
          title: updatedLog.title,
          content: updatedLog.content,
          tags: updatedLog.tags
        })
        logs.value[index] = updatedLog
      } catch (error) {
        console.error('Failed to update log:', error)
      }
    }
  }

  async function deleteLog(id: string) {
    try {
      await invoke('delete_log', { id })
      logs.value = logs.value.filter(l => l.id !== id)
      if (currentLog.value?.id === id) {
        currentLog.value = logs.value.length > 0 ? logs.value[0] : null
      }
    } catch (error) {
      console.error('Failed to delete log:', error)
    }
  }

  async function searchLogs(keyword: string) {
    if (!keyword.trim()) {
      return fetchLogs()
    }
    isLoading.value = true
    try {
      logs.value = await invoke<LogEntry[]>('search_logs', { keyword })
    } catch (error) {
      console.error('Failed to search logs:', error)
    } finally {
      isLoading.value = false
    }
  }

  function reorderLogs(newLogs: LogEntry[]) {
    logs.value = newLogs
  }

  function sortLogsByDate(order: 'asc' | 'desc' = 'desc') {
    logs.value.sort((a, b) => {
      const dateA = new Date(a.date).getTime()
      const dateB = new Date(b.date).getTime()
      return order === 'desc' ? dateB - dateA : dateA - dateB
    })
  }

  return { 
    logs, 
    filteredLogs,
    currentLog, 
    isLoading, 
    showSnapshot, 
    filterDate,
    fetchLogs, 
    addLog, 
    updateLog, 
    deleteLog, 
    searchLogs,
    reorderLogs,
    sortLogsByDate
  }
})
