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
        <n-layout has-sider class="h-screen overflow-hidden bg-crayon-bg">
        <!-- Sidebar -->
        <n-layout-sider
          v-if="!isMobile || !collapsedSidebar"
          collapse-mode="width"
          :collapsed-width="isMobile ? 0 : 64"
          :width="isMobile ? '100%' : 280"
          :collapsed="collapsedSidebar"
          show-trigger
          resizable
          class="h-full z-20 border-r-2 border-gray-800"
          @collapse="collapsedSidebar = true"
          @expand="collapsedSidebar = false"
        >
          <Sidebar />
        </n-layout-sider>

        <!-- Main Content -->
        <n-layout class="h-full bg-transparent">
          <n-layout-header class="p-4 flex justify-between items-center h-16 bg-white/80 backdrop-blur-md border-b-2 border-gray-800 z-10">
            <div class="flex items-center space-x-2">
              <n-button v-if="isMobile" quaternary circle @click="collapsedSidebar = !collapsedSidebar">
                <template #icon>
                  <n-icon><SettingsOutline /></n-icon>
                </template>
              </n-button>
              <div class="flex items-center space-x-2">
                <div class="w-8 h-8 text-crayon-pink animate-float">
                  <svg viewBox="0 0 100 100" fill="currentColor">
                    <path d="M50 80c-10 0-25-10-25-25 0-15 15-20 25-10 10-10 25-5 25 10 0 15-15 25-25 25z" />
                  </svg>
                </div>
                <h1 class="text-xl md:text-2xl font-bold font-hand text-gray-800">智能工作日志</h1>
              </div>
            </div>
            <div class="flex items-center space-x-1 md:space-x-2">
              <n-button quaternary circle @click="settingsStore.toggleDarkMode" class="!text-gray-600">
                <template #icon>
                  <n-icon><SunnyOutline v-if="settingsStore.isDarkMode" /><MoonOutline v-else /></n-icon>
                </template>
              </n-button>
              <n-button quaternary circle @click="showSettings = true" class="!text-gray-600">
                <template #icon>
                  <n-icon><SettingsOutline /></n-icon>
                </template>
              </n-button>
              <n-button 
                quaternary 
                circle 
                @click="showAIPanel = !showAIPanel" 
                :class="showAIPanel ? '!bg-crayon-purple !text-white' : '!text-gray-600'"
                class="shadow-sm transition-all"
              >
                <template #icon>
                  <n-icon><ChatbubbleEllipsesOutline /></n-icon>
                </template>
              </n-button>
            </div>
          </n-layout-header>

          <n-layout has-sider position="absolute" style="top: 64px; bottom: 0" class="bg-transparent">
            <n-layout-content content-style="padding: 0;" class="bg-transparent overflow-y-auto">
              <Editor />
            </n-layout-content>

            <n-layout-sider
              v-if="showAIPanel && !isMobile"
              width="350"
              class="h-full border-l-2 border-gray-800 shadow-xl"
            >
              <AIPanel />
            </n-layout-sider>
          </n-layout>
        </n-layout>
      </n-layout>

      <!-- Mobile AI Drawer -->
        <div 
          v-if="showAIPanel && isMobile" 
          class="fixed inset-0 z-50 bg-black/20 backdrop-blur-sm"
          @click="showAIPanel = false"
        >
          <div 
            class="absolute right-0 top-0 bottom-0 w-[85%] bg-white shadow-2xl"
            @click.stop
          >
            <AIPanel />
            <n-button 
              quaternary 
              circle 
              class="absolute top-2 right-2 z-50"
              @click="showAIPanel = false"
            >
              <template #icon><n-icon><CloseOutline /></n-icon></template>
            </n-button>
          </div>
        </div>

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
