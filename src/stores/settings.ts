import { defineStore } from 'pinia'
import { ref } from 'vue'
import { LazyStore } from '@tauri-apps/plugin-store'

const store = new LazyStore('settings.json')

export const useSettingsStore = defineStore('settings', () => {
  const isDarkMode = ref(false)
  const apiKey = ref('')
  const apiEndpoint = ref('https://api.deepseek.com/v1')
  const model = ref('deepseek-chat')
  const isLoaded = ref(false)

  async function loadSettings() {
    const savedDarkMode = await store.get<boolean>('isDarkMode')
    if (savedDarkMode !== undefined) isDarkMode.value = savedDarkMode
    
    const savedApiKey = await store.get<string>('apiKey')
    if (savedApiKey !== undefined) apiKey.value = savedApiKey
    
    const savedApiEndpoint = await store.get<string>('apiEndpoint')
    if (savedApiEndpoint !== undefined) apiEndpoint.value = savedApiEndpoint

    const savedModel = await store.get<string>('model')
    if (savedModel !== undefined) model.value = savedModel

    if (isDarkMode.value) {
      document.documentElement.classList.add('dark')
    }
    isLoaded.value = true
  }

  async function saveSettings() {
    await store.set('isDarkMode', isDarkMode.value)
    await store.set('apiKey', apiKey.value)
    await store.set('apiEndpoint', apiEndpoint.value)
    await store.set('model', model.value)
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

  return { isDarkMode, apiKey, apiEndpoint, model, isLoaded, loadSettings, saveSettings, toggleDarkMode }
})
