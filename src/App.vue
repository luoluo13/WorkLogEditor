<script setup lang="ts">
import { NConfigProvider, NLayout, NLayoutHeader, NMessageProvider, NNotificationProvider, NButton, NIcon, NSplit, darkTheme } from 'naive-ui'
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
const sidebarSplitSize = ref(0.24)
const editorSplitSize = ref(0.74)

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
        <n-layout class="h-screen overflow-hidden bg-crayon-bg">
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
                <h1 class="text-xl md:text-2xl font-bold font-hand text-gray-800">WorkLogEditor</h1>
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

          <div class="h-[calc(100vh-64px)] bg-transparent">
            <div v-if="isMobile" class="h-full bg-transparent overflow-y-auto">
              <Editor />
            </div>

            <n-split
              v-else
              direction="horizontal"
              :default-size="sidebarSplitSize"
              :size="sidebarSplitSize"
              :min="0.16"
              :max="0.42"
              :resize-trigger-size="10"
              class="h-full app-shell-split"
              @update:size="sidebarSplitSize = $event"
            >
              <template #1>
                <div class="h-full border-r-2 border-gray-800 bg-crayon-bg">
                  <Sidebar />
                </div>
              </template>

              <template #2>
                <n-split
                  v-if="showAIPanel"
                  direction="horizontal"
                  :default-size="editorSplitSize"
                  :size="editorSplitSize"
                  :min="0.45"
                  :max="0.85"
                  :resize-trigger-size="10"
                  class="h-full app-shell-split"
                  @update:size="editorSplitSize = $event"
                >
                  <template #1>
                    <div class="h-full bg-transparent overflow-y-auto">
                      <Editor />
                    </div>
                  </template>

                  <template #2>
                    <div class="h-full border-l-2 border-gray-800 shadow-xl bg-white/70">
                      <AIPanel />
                    </div>
                  </template>
                </n-split>

                <div v-else class="h-full bg-transparent overflow-y-auto">
                  <Editor />
                </div>
              </template>
            </n-split>
          </div>
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

        <div
          v-if="isMobile && !collapsedSidebar"
          class="fixed inset-0 z-50 bg-black/20 backdrop-blur-sm"
          @click="collapsedSidebar = true"
        >
          <div
            class="absolute left-0 top-0 bottom-0 w-[85%] bg-white shadow-2xl"
            @click.stop
          >
            <Sidebar />
            <n-button
              quaternary
              circle
              class="absolute top-2 right-2 z-50"
              @click="collapsedSidebar = true"
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

.app-shell-split :deep(.n-split__resize-trigger-wrapper) {
  width: 8px !important;
  margin-left: -4px;
  margin-right: -4px;
  position: relative;
  z-index: 10;
  cursor: col-resize;
  background: transparent !important;
}

.app-shell-split :deep(.n-split__resize-trigger) {
  width: 3px !important;
  height: 100%;
  margin: 0 auto;
  background-color: transparent !important;
  border-radius: 9999px;
  transition: background-color 0.2s ease;
}

.app-shell-split :deep(.n-split__resize-trigger:hover),
.app-shell-split :deep(.n-split__resize-trigger--hover) {
  background-color: transparent !important;
}
</style>
