import { defineStore } from 'pinia'
import { ref } from 'vue'
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

  return { logs, currentLog, isLoading, fetchLogs, addLog, updateLog, deleteLog, searchLogs }
})
