<script setup lang="ts">
import {RouterView} from 'vue-router'
import {computed, onMounted, onUnmounted, ref, watch} from 'vue'
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

// Re-show the alert whenever a new error arrives
watch(() => store.error, (e) => { if (e) dismissed.value = false })

let timer: ReturnType<typeof setInterval> | null = null

onMounted(() => {
  void store.refresh()
  timer = setInterval(() => void store.refresh(), 5000)
})

onUnmounted(() => {
  if (timer) clearInterval(timer)
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
      <RouterView/>
    </main>
  </div>
</template>

<style scoped>
.refresh-alert {
  margin-bottom: 1rem;
}
</style>
