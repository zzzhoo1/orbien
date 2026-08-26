<script setup lang="ts">
import {computed, onMounted, onUnmounted, ref, watch} from 'vue'
import {RouterView} from 'vue-router'
import AppHeader from '@/layouts/AppHeader.vue'
import AppSidebar from '@/layouts/AppSidebar.vue'
import InlineAlert from '@/components/InlineAlert.vue'
import {useDashboardStore} from '@/stores/dashboard'
import {useLocale} from '@/composables/useLocale'
import {useSidebar} from '@/composables/useSidebar'

const store = useDashboardStore()
const {t} = useLocale()
const {desktopCollapsed, isMobile} = useSidebar()

const dismissed = ref(false)

const errorText = computed(() => {
  const err = store.error
  if (!err) return ''
  if (err.code === 'http') {
    return t('errors.http', err.params ?? {})
  }
  if (err.code === 'api' && typeof err.params?.msg === 'string' && err.params.msg) {
    return err.params.msg
  }
  return t(`errors.${err.code}`)
})

// Global toast/message bus
const toast = ref<{ id: number; type: 'info' | 'error'; text: string } | null>(null)
let toastTimer: number | null = null
let nextToastId = 1

function showToast(type: 'info' | 'error', text: string, duration = 3000) {
  toast.value = {id: nextToastId++, type, text}
  if (toastTimer !== null) {
    window.clearTimeout(toastTimer)
  }
  toastTimer = window.setTimeout(() => {
    toast.value = null
    toastTimer = null
  }, duration)
}

// Expose on window for now so views can call window.__orbienToast?.(...)
// Later this can be refactored into a dedicated composable/store.
window.__orbienToast = showToast

// Re-show the alert whenever a new error arrives
watch(() => store.error, (e) => { if (e) dismissed.value = false })

let timer: ReturnType<typeof setInterval> | null = null

onMounted(() => {
  void store.refresh()
  timer = setInterval(() => void store.refresh(), 5000)
})

onUnmounted(() => {
  if (timer) clearInterval(timer)
  if (toastTimer !== null) {
    window.clearTimeout(toastTimer)
    toastTimer = null
  }
})
</script>

<template>
  <div
      class="shell"
      :class="{
      'sidebar-collapsed': desktopCollapsed,
      'sidebar-mobile': isMobile,
    }"
  >
    <AppHeader/>
    <AppSidebar/>
    <main class="content">
      <InlineAlert
        v-if="errorText && !dismissed"
        variant="error"
        :title="errorText"
        :closable="true"
        class="refresh-alert"
        @close="dismissed = true"
      />

      <div
          v-if="toast"
          class="global-toast"
          :class="toast.type"
          role="status"
          aria-live="polite"
      >
        {{ toast.text }}
      </div>

      <RouterView/>
    </main>
  </div>
</template>

<style scoped>
.refresh-alert {
  margin-bottom: 1rem;
}

.global-toast {
  margin-bottom: 0.75rem;
  padding: 0.5rem 0.9rem;
  border-radius: var(--radius-pill);
  font-size: 0.8rem;
  font-weight: 600;
  background: color-mix(in srgb, var(--accent) 14%, transparent);
  border: 1px solid color-mix(in srgb, var(--accent) 26%, transparent);
  color: var(--accent-text);
  box-shadow: var(--shadow);
}

.global-toast.error {
  background: color-mix(in srgb, var(--danger, #ef4444) 16%, transparent);
  border-color: color-mix(in srgb, var(--danger, #ef4444) 30%, transparent);
  color: #fed7d7;
}
</style>
