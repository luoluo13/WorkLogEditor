import { defineStore } from 'pinia'
import { ref } from 'vue'
import { LazyStore } from '@tauri-apps/plugin-store'
import { invoke } from '@tauri-apps/api/core'

const store = new LazyStore('settings.json')

export const useSettingsStore = defineStore('settings', () => {
  const isDarkMode = ref(false)
  const provider = ref<'deepseek' | 'ollama'>('deepseek')
  const apiKey = ref('')
  const apiEndpoint = ref('https://api.deepseek.com/v1')
  const ollamaEndpoint = ref('http://localhost:11434')
  const model = ref('deepseek-chat')
  const ollamaModel = ref('')
  const availableOllamaModels = ref<string[]>([])
  const isLoaded = ref(false)

  async function loadSettings() {
    const savedDarkMode = await store.get<boolean>('isDarkMode')
    if (savedDarkMode !== undefined) isDarkMode.value = savedDarkMode
    
    const savedProvider = await store.get<'deepseek' | 'ollama'>('provider')
    if (savedProvider !== undefined) provider.value = savedProvider

    const savedApiKey = await store.get<string>('apiKey')
    if (savedApiKey !== undefined) apiKey.value = savedApiKey
    
    const savedApiEndpoint = await store.get<string>('apiEndpoint')
    if (savedApiEndpoint !== undefined) apiEndpoint.value = savedApiEndpoint

    const savedOllamaEndpoint = await store.get<string>('ollamaEndpoint')
    if (savedOllamaEndpoint !== undefined) ollamaEndpoint.value = savedOllamaEndpoint

    const savedModel = await store.get<string>('model')
    if (savedModel !== undefined) model.value = savedModel

    const savedOllamaModel = await store.get<string>('ollamaModel')
    if (savedOllamaModel !== undefined) ollamaModel.value = savedOllamaModel

    if (isDarkMode.value) {
      document.documentElement.classList.add('dark')
    }
    
    if (provider.value === 'ollama') {
      await fetchOllamaModels()
    }

    isLoaded.value = true
  }

  async function fetchOllamaModels() {
    try {
      availableOllamaModels.value = await invoke<string[]>('get_ollama_models', { baseUrl: ollamaEndpoint.value })
      if (availableOllamaModels.value.length > 0 && !ollamaModel.value) {
        ollamaModel.value = availableOllamaModels.value[0]
      }
    } catch (error) {
      console.error('Failed to fetch Ollama models:', error)
      availableOllamaModels.value = []
    }
  }

  async function saveSettings() {
    await store.set('isDarkMode', isDarkMode.value)
    await store.set('provider', provider.value)
    await store.set('apiKey', apiKey.value)
    await store.set('apiEndpoint', apiEndpoint.value)
    await store.set('ollamaEndpoint', ollamaEndpoint.value)
    await store.set('model', model.value)
    await store.set('ollamaModel', ollamaModel.value)
    await store.save()
  }

  function toggleDarkMode() {
    isDarkMode.value = !isDarkMode.value
    if (isDarkMode.value) {
      document.documentElement.classList.add('dark')
    } else {
      document.documentElement.classList.remove('dark')
    }
    saveSettings()
  }

  return { 
    isDarkMode, 
    provider,
    apiKey, 
    apiEndpoint, 
    ollamaEndpoint,
    model, 
    ollamaModel,
    availableOllamaModels,
    isLoaded, 
    loadSettings, 
    saveSettings, 
    toggleDarkMode,
    fetchOllamaModels
  }
})
