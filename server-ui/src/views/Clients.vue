<script setup lang="ts">
import {computed, ref, watch} from 'vue'
import {useRouter} from 'vue-router'
import AppIcon from '@/components/AppIcon.vue'
import OsBadge from '@/components/OsBadge.vue'
import PaginationBar from '@/components/PaginationBar.vue'
import {kickClient} from '@/api'
import {useDashboardStore} from '@/stores/dashboard'
import {useLocale} from '@/composables/useLocale'
import {usePresence} from '@/composables/usePresence'
import signalIcon from '@/assets/icon/signal.svg?raw'

type StatusFilter = 'all' | 'online' | 'offline'

const FILTERS: StatusFilter[] = ['all', 'online', 'offline']

const store = useDashboardStore()
const router = useRouter()
const {t} = useLocale()
const {isOnline, statusLabel, formatSeen} = usePresence()

const page = ref(1)
const pageSize = ref(10)
const statusFilter = ref<StatusFilter>('all')
const kicking = ref<string | null>(null)

const filtered = computed(() => {
  const list = store.clients
  if (statusFilter.value === 'all') return list
  if (statusFilter.value === 'online') return list.filter((c) => isOnline(c.status))
  return list.filter((c) => !isOnline(c.status))
})

const total = computed(() => filtered.value.length)

const pageItems = computed(() => {
  const start = (page.value - 1) * pageSize.value
  return filtered.value.slice(start, start + pageSize.value)
})

const statusCounts = computed(() => {
  let online = 0
  let offline = 0
  for (const c of store.clients) {
    if (isOnline(c.status)) online += 1
    else offline += 1
  }
  return {
    all: store.clients.length,
    online,
    offline,
  } as Record<StatusFilter, number>
})

watch([total, pageSize, statusFilter], () => {
  const maxPage = Math.max(1, Math.ceil(total.value / Math.max(pageSize.value, 1)))
  if (page.value > maxPage) page.value = maxPage
})

watch(statusFilter, () => {
  page.value = 1
})

function filterLabel(key: StatusFilter) {
  if (key === 'all') return t('clients.filterAll')
  if (key === 'online') return t('status.online')
  return t('status.offline')
}

function openDetail(sessionId: string) {
  router.push({name: 'client-detail', params: {sessionId}})
}

function onKeyOpen(evt: KeyboardEvent, sessionId: string) {
  if (evt.key === 'Enter' || evt.key === ' ') {
    evt.preventDefault()
    openDetail(sessionId)
  }
}

async function onKick(sessionId: string, evt: Event) {
  evt.stopPropagation()
  if (kicking.value) return
  if (!window.confirm(t('clients.kickConfirm'))) return
  kicking.value = sessionId
  try {
    await kickClient(sessionId)
    await store.refresh()
  } catch (e) {
    const msg = e instanceof Error ? e.message : String(e)
    window.alert(t('clients.kickFailed', {msg}))
  } finally {
    kicking.value = null
  }
}
</script>

<template>
  <section class="client-list">
    <div class="list-toolbar" role="group" :aria-label="t('clients.filter')">
      <button
          v-for="key in FILTERS"
          :key="key"
          type="button"
          class="filter-chip"
          :class="{ active: statusFilter === key }"
          @click="statusFilter = key"
      >
        <span>{{ filterLabel(key) }}</span>
        <em>{{ statusCounts[key] }}</em>
      </button>
    </div>

    <div v-if="!store.clients.length" class="empty-card">
      {{ t('clients.empty') }}
    </div>
    <div v-else-if="!filtered.length" class="empty-card">
      {{ t('clients.filterEmpty') }}
    </div>

    <article
        v-for="c in pageItems"
        :key="c.sessionId"
        class="client-card"
        :class="{ offline: !isOnline(c.status) }"
        role="button"
        tabindex="0"
        :aria-label="t('clients.detail')"
        @click="openDetail(c.sessionId)"
        @keydown="onKeyOpen($event, c.sessionId)"
    >
      <div class="client-left">
        <div class="os-avatar" :class="{ online: isOnline(c.status) }" aria-hidden="true">
          <OsBadge :os="c.os" :arch="c.arch" icon-only/>
          <span class="dot"/>
        </div>

        <div class="client-body">
          <div class="client-title">
            <h3 class="client-id">{{ c.sessionId }}</h3>
            <span v-if="c.hostname" class="tag">{{ c.hostname }}</span>
            <span v-if="c.user" class="tag">{{ c.user }}</span>
            <span v-if="c.version" class="tag version">v{{ c.version }}</span>
            <span class="tag soft">
              {{ t('clients.tunnels') }} {{ c.tunnelCount ?? 0 }}
            </span>
          </div>
          <div class="client-meta">
            <span>
              {{ t('clients.ip') }}
              <strong class="mono">{{ c.clientIP || '—' }}</strong>
            </span>
            <OsBadge :os="c.os" :arch="c.arch" text-only/>
            <span class="seen">
              <span class="seen-icon" aria-hidden="true" v-html="signalIcon"/>
              {{ formatSeen(c.connectedSecs, isOnline(c.status)) }}
            </span>
          </div>
        </div>
      </div>

      <div class="client-right">
        <button
            v-if="isOnline(c.status)"
            type="button"
            class="kick-btn"
            :disabled="kicking === c.sessionId"
            :title="t('clients.kick')"
            :aria-label="t('clients.kick')"
            @click="onKick(c.sessionId, $event)"
        >
          <AppIcon name="kick"/>
        </button>
        <span class="status-badge" :class="{ online: isOnline(c.status) }">
          {{ statusLabel(c.status) }}
        </span>
      </div>
    </article>

    <PaginationBar
        v-model:page="page"
        v-model:page-size="pageSize"
        :total="total"
    />
  </section>
</template>

<style scoped>
.client-list {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}

.list-toolbar {
  display: flex;
  flex-wrap: wrap;
  gap: 0.45rem;
}

.filter-chip {
  display: inline-flex;
  align-items: center;
  gap: 0.4rem;
  border: 1px solid var(--line);
  background: var(--panel);
  color: var(--muted);
  font: inherit;
  font-size: 0.78rem;
  font-weight: 650;
  padding: 0.38rem 0.75rem;
  border-radius: var(--radius-pill);
  cursor: pointer;
  box-shadow: var(--shadow);
  transition: border-color 0.15s ease,
  color 0.15s ease,
  background 0.15s ease;
}

.filter-chip em {
  font-style: normal;
  font-variant-numeric: tabular-nums;
  font-size: 0.72rem;
  font-weight: 600;
  min-width: 1.1rem;
  padding: 0.05rem 0.4rem;
  border-radius: var(--radius-pill);
  text-align: center;
  color: var(--muted);
  background: color-mix(in srgb, var(--muted) 12%, transparent);
}

.filter-chip:hover:not(.active) {
  color: var(--text);
  border-color: var(--line-strong);
}

.filter-chip.active {
  color: var(--accent-text);
  border-color: color-mix(in srgb, var(--accent) 35%, transparent);
  background: var(--accent-soft);
}

.filter-chip.active em {
  color: var(--accent-text);
  background: color-mix(in srgb, var(--accent) 18%, transparent);
}

.empty-card {
  padding: 2.5rem 1rem;
  text-align: center;
  color: var(--muted);
  background: var(--panel);
  border: 1px solid var(--line);
  border-radius: var(--radius);
  box-shadow: var(--shadow);
}

.client-card {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 1rem;
  padding: 1rem 1.15rem;
  background: var(--panel);
  border: 1px solid var(--line);
  border-radius: var(--radius);
  box-shadow: var(--shadow);
  cursor: pointer;
  transition: border-color 0.18s ease, box-shadow 0.18s ease, background 0.18s ease;
}

.client-card:hover {
  border-color: var(--line-strong);
  background: var(--panel-hover);
}

.client-card:focus-visible {
  outline: none;
  box-shadow: var(--focus-ring);
}

.client-card.offline {
  opacity: 0.92;
}

.client-left {
  display: flex;
  align-items: flex-start;
  gap: 0.85rem;
  min-width: 0;
  flex: 1;
}

.os-avatar {
  position: relative;
  width: 2.45rem;
  height: 2.45rem;
  border-radius: var(--radius);
  display: grid;
  place-items: center;
  flex-shrink: 0;
  background: color-mix(in srgb, var(--muted) 10%, transparent);
  border: 1px solid color-mix(in srgb, var(--muted) 14%, transparent);
}

.os-avatar.online {
  background: var(--accent-soft);
  border-color: color-mix(in srgb, var(--accent) 22%, transparent);
}

.os-avatar :deep(.os-icon) {
  width: 1.25rem;
  height: 1.25rem;
}

.os-avatar .dot {
  position: absolute;
  right: -0.12rem;
  bottom: -0.12rem;
  width: 0.52rem;
  height: 0.52rem;
  border-radius: var(--radius-circle);
  background: var(--muted);
  border: 2px solid var(--panel);
  box-sizing: content-box;
}

.os-avatar.online .dot {
  background: var(--accent);
  box-shadow: 0 0 0 2px color-mix(in srgb, var(--accent) 18%, transparent);
  animation: pulse-dot 2s ease-in-out infinite;
}

@keyframes pulse-dot {
  0%,
  100% {
    box-shadow: 0 0 0 2px color-mix(in srgb, var(--accent) 18%, transparent);
    opacity: 1;
  }
  50% {
    box-shadow: 0 0 0 4px color-mix(in srgb, var(--accent) 8%, transparent);
    opacity: 0.85;
  }
}

.client-body {
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 0.4rem;
}

.client-title {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 0.4rem;
}

.client-id {
  margin: 0;
  font-size: 1rem;
  font-weight: 700;
  font-family: 'IBM Plex Mono', ui-monospace, monospace;
  letter-spacing: -0.01em;
  color: var(--text);
}

.tag {
  display: inline-flex;
  align-items: center;
  padding: 0.12rem 0.5rem;
  border-radius: var(--radius-pill);
  font-size: 0.72rem;
  font-weight: 600;
  color: var(--muted);
  background: color-mix(in srgb, var(--muted) 10%, transparent);
  border: 1px solid color-mix(in srgb, var(--muted) 14%, transparent);
}

.tag.version {
  color: var(--accent-text);
  background: var(--accent-soft);
  border-color: color-mix(in srgb, var(--accent) 22%, transparent);
}

.tag.soft {
  font-weight: 550;
}

.client-meta {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 0.75rem 1.1rem;
  font-size: 0.8rem;
  color: var(--muted);
}

.client-meta strong {
  font-weight: 600;
  color: var(--text-secondary, var(--text));
  margin-left: 0.25rem;
}

.mono {
  font-family: 'IBM Plex Mono', ui-monospace, monospace;
  font-size: 0.78rem;
}

.seen {
  display: inline-flex;
  align-items: center;
  gap: 0.3rem;
}

.seen-icon {
  width: 0.82rem;
  height: 0.82rem;
  display: inline-grid;
  place-items: center;
  line-height: 0;
  color: inherit;
  opacity: 0.85;
  flex-shrink: 0;
}

.seen-icon :deep(svg) {
  width: 100%;
  height: 100%;
  display: block;
  fill: currentColor;
  stroke: none;
}

.client-right {
  display: flex;
  align-items: center;
  gap: 0.65rem;
  flex-shrink: 0;
}

.status-badge {
  box-sizing: border-box;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 3.8rem;
  min-height: 1.85rem;
  padding: 0.28rem 0.75rem;
  border-radius: var(--radius-pill);
  font-size: 0.78rem;
  font-weight: 650;
  line-height: 1.25;
  color: var(--muted);
  background: color-mix(in srgb, var(--muted) 12%, transparent);
  border: 1px solid color-mix(in srgb, var(--muted) 18%, transparent);
}

.kick-btn {
  box-sizing: border-box;
  width: 1.85rem;
  height: 1.85rem;
  padding: 0;
  border-radius: var(--radius);
  border: 1px solid color-mix(in srgb, var(--danger, #ef4444) 45%, transparent);
  background: transparent;
  color: var(--danger, #ef4444);
  display: inline-grid;
  place-items: center;
  cursor: pointer;
  font-size: 1rem;
  transition: border-color 0.15s ease;
}

.kick-btn:hover:not(:disabled) {
  border-color: var(--danger, #ef4444);
}

.kick-btn:disabled {
  opacity: 0.45;
  cursor: wait;
}

.status-badge.online {
  color: var(--status-ok);
  background: var(--status-ok-soft);
  border-color: color-mix(in srgb, var(--status-ok) 22%, transparent);
}

@media (max-width: 720px) {
  .client-card {
    flex-direction: column;
    align-items: stretch;
  }

  .client-right {
    align-self: flex-end;
  }
}
</style>
