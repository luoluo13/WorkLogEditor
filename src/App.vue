<script setup lang="ts">
import { NConfigProvider, NLayout, NLayoutSider, NLayoutContent, NLayoutHeader, NMessageProvider, NNotificationProvider, NButton, NIcon, darkTheme } from 'naive-ui'
import { MoonOutline, SunnyOutline, SettingsOutline, ChatbubbleEllipsesOutline, CloseOutline } from '@vicons/ionicons5'
import { useSettingsStore } from './stores/settings'
import { useLogStore } from './stores/log'
import Sidebar from './components/Sidebar.vue'
import Editor from './components/Editor.vue'
import AIPanel from './components/AIPanel.vue'
import SettingsModal from './components/SettingsModal.vue'
import { ref, computed, onMounted, onUnmounted } from 'vue'

const settingsStore = useSettingsStore()
const logStore = useLogStore()
const theme = computed(() => settingsStore.isDarkMode ? darkTheme : null)

const showAIPanel = ref(true)
const showSettings = ref(false)
const isMobile = ref(window.innerWidth < 768)
const collapsedSidebar = ref(window.innerWidth < 1024)

const handleResize = () => {
  isMobile.value = window.innerWidth < 768
  if (isMobile.value) {
    showAIPanel.value = false
    collapsedSidebar.value = true
  }
}

onMounted(async () => {
  try {
    window.addEventListener('resize', handleResize)
    handleResize()
    await settingsStore.loadSettings()
    await logStore.fetchLogs()
  } catch (err) {
    console.error('Failed to initialize app:', err)
  }
})

onUnmounted(() => {
  window.removeEventListener('resize', handleResize)
})
</script>

<template>
  <n-config-provider :theme="theme">
    <n-notification-provider>
      <n-message-provider>
        <n-layout has-sider class="h-screen overflow-hidden">
        <!-- Sidebar -->
        <n-layout-sider
          v-if="!isMobile || !collapsedSidebar"
          bordered
          collapse-mode="width"
          :collapsed-width="isMobile ? 0 : 64"
          :width="isMobile ? '100%' : 260"
          :collapsed="collapsedSidebar"
          show-trigger
          resizable
          class="h-full z-20"
          @collapse="collapsedSidebar = true"
          @expand="collapsedSidebar = false"
        >
          <Sidebar />
        </n-layout-sider>

        <!-- Main Content -->
        <n-layout class="h-full">
          <n-layout-header bordered class="p-4 flex justify-between items-center h-16">
            <div class="flex items-center space-x-2">
              <n-button v-if="isMobile" quaternary circle @click="collapsedSidebar = !collapsedSidebar">
                <template #icon>
                  <n-icon><SettingsOutline /></n-icon>
                </template>
              </n-button>
              <h1 class="text-lg md:text-xl font-bold truncate">智能工作日志</h1>
            </div>
            <div class="flex items-center space-x-1 md:space-x-2">
              <n-button quaternary circle @click="settingsStore.toggleDarkMode">
                <template #icon>
                  <n-icon><SunnyOutline v-if="settingsStore.isDarkMode" /><MoonOutline v-else /></n-icon>
                </template>
              </n-button>
              <n-button quaternary circle @click="showSettings = true">
                <template #icon>
                  <n-icon><SettingsOutline /></n-icon>
                </template>
              </n-button>
              <n-button quaternary circle @click="showAIPanel = !showAIPanel" :type="showAIPanel ? 'primary' : 'default'">
                <template #icon>
                  <n-icon><ChatbubbleEllipsesOutline /></n-icon>
                </template>
              </n-button>
            </div>
          </n-layout-header>

          <n-layout has-sider position="absolute" style="top: 64px; bottom: 0">
            <n-layout-content content-style="padding: 12px;" class="bg-gray-50 dark:bg-zinc-900 overflow-y-auto md:p-6">
              <Editor />
            </n-layout-content>
            
            <n-layout-sider
              v-if="showAIPanel && !isMobile"
              bordered
              width="300"
              resizable
              class="h-full"
            >
              <AIPanel />
            </n-layout-sider>
            
            <!-- Mobile AI Drawer-like Overlay -->
            <div v-if="showAIPanel && isMobile" class="absolute inset-0 z-30 bg-white dark:bg-zinc-900">
              <div class="flex justify-between items-center p-4 border-b dark:border-zinc-800">
                <span class="font-bold">AI 助手</span>
                <n-button quaternary circle @click="showAIPanel = false">
                  <template #icon><n-icon><CloseOutline /></n-icon></template>
                </n-button>
              </div>
              <AIPanel />
            </div>
          </n-layout>
        </n-layout>
      </n-layout>
      <SettingsModal v-model:show="showSettings" />
    </n-message-provider>
  </n-notification-provider>
</n-config-provider>
</template>

<style>
.n-layout-header {
  background-color: transparent;
}
</style>
