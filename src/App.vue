<script setup lang="ts">
import { NConfigProvider, NLayout, NLayoutSider, NLayoutContent, NLayoutHeader, NMessageProvider, NNotificationProvider, NButton, NIcon, darkTheme } from 'naive-ui'
import { MoonOutline, SunnyOutline, SettingsOutline, ChatbubbleEllipsesOutline } from '@vicons/ionicons5'
import { useSettingsStore } from './stores/settings'
import { useLogStore } from './stores/log'
import Sidebar from './components/Sidebar.vue'
import Editor from './components/Editor.vue'
import AIPanel from './components/AIPanel.vue'
import SettingsModal from './components/SettingsModal.vue'
import { ref, computed, onMounted } from 'vue'

const settingsStore = useSettingsStore()
const logStore = useLogStore()
const theme = computed(() => settingsStore.isDarkMode ? darkTheme : null)

const showAIPanel = ref(true)
const showSettings = ref(false)

onMounted(async () => {
  await settingsStore.loadSettings()
  logStore.fetchLogs()
})
</script>

<template>
  <n-config-provider :theme="theme">
    <n-notification-provider>
      <n-message-provider>
        <n-layout has-sider class="h-screen">
        <!-- Sidebar -->
        <n-layout-sider
          bordered
          collapse-mode="width"
          :collapsed-width="64"
          :width="260"
          show-trigger
          resizable
          class="h-full"
        >
          <Sidebar />
        </n-layout-sider>

        <!-- Main Content -->
        <n-layout class="h-full">
          <n-layout-header bordered class="p-4 flex justify-between items-center h-16">
            <div class="flex items-center space-x-4">
              <h1 class="text-xl font-bold">智能工作日志</h1>
            </div>
            <div class="flex items-center space-x-2">
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
            <n-layout-content content-style="padding: 24px;" class="bg-gray-50 dark:bg-zinc-900">
              <Editor />
            </n-layout-content>
            
            <n-layout-sider
              v-if="showAIPanel"
              bordered
              width="300"
              resizable
              class="h-full"
            >
              <AIPanel />
            </n-layout-sider>
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
