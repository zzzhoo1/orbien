<script setup lang="ts">
import {computed, onMounted, onUnmounted, ref, watch} from 'vue'
import {useRoute, useRouter} from 'vue-router'
import AppIcon from '@/components/AppIcon.vue'
import PaginationBar from '@/components/PaginationBar.vue'
import SectionCard from '@/components/SectionCard.vue'
import StatusBadge from '@/components/StatusBadge.vue'
import TrafficChart from '@/components/TrafficChart.vue'
import TrafficIO from '@/components/TrafficIO.vue'
import type {TrafficRange} from '@/api'
import {fetchConnections, kickProxy} from '@/api'
import type {ConnectionInfo} from '@/types/api'
import {useDashboardStore} from '@/stores/dashboard'
import {useLocale} from '@/composables/useLocale'
import {usePresence} from '@/composables/usePresence'
import {useToast} from '@/composables/useToast'
import {formatTunnelEndpoint, isHttpTunnelType} from '@/utils/format'

const route = useRoute()
const router = useRouter()
const store = useDashboardStore()
const {t} = useLocale()
const {isOnline, statusLabel} = usePresence()
const {show: showToast} = useToast()
const trafficRange = ref<TrafficRange>('24h')
const chartVariant = ref<'bar' | 'line'>('bar')
const name = computed(() => String(route.params.name || ''))
const tunnel = computed(() => store.tunnels.find((t) => t.name === name.value) || null)

// ── delete state ──────────────────────────────────────────────────────────────
const confirmDelete = ref(false)
const deleting = ref(false)

// ── connections panel ─────────────────────────────────────────────────────────
const connections = ref<ConnectionInfo[]>([])
const connTotal = ref(0)
const connPage = ref(1)
const connPageSize = ref(10)
const connSearch = ref('')
const connLoading = ref(false)
const connReady = ref(false)
let connReqSeq = 0
let connSearchDebounce: number | null = null
let connRefreshTimer: number | null = null

async function loadConnections() {
  const seq = ++connReqSeq
  connLoading.value = true
  try {
    const q = connSearch.value.trim()
    const data = await fetchConnections(name.value, {
      page: connPage.value,
      pageSize: connPageSize.value,
      q: q || undefined,
    })
    if (seq !== connReqSeq) return
    const maxPage = Math.max(1, Math.ceil(data.total / Math.max(data.pageSize, 1)))
    if (data.items.length === 0 && data.total > 0 && data.page > maxPage) {
      connPage.value = maxPage
      await loadConnections()
      return
    }
    connections.value = data.items ?? []
    connTotal.value = data.total
    connPage.value = data.page
    connPageSize.value = data.pageSize
  } catch {
    if (seq !== connReqSeq) return
  } finally {
    if (seq === connReqSeq) connLoading.value = false
  }
}

function clearConnSearchDebounce() {
  if (connSearchDebounce !== null) {
    window.clearTimeout(connSearchDebounce)
    connSearchDebounce = null
  }
}

watch(connSearch, () => {
  if (!connReady.value) return
  clearConnSearchDebounce()
  connReqSeq++
  connLoading.value = false
  connPage.value = 1
  connSearchDebounce = window.setTimeout(() => {
    connSearchDebounce = null
    loadConnections()
  }, 300)
})

watch([connPage, connPageSize], () => {
  if (!connReady.value) return
  if (connSearchDebounce !== null) return
  loadConnections()
})

onMounted(async () => {
  await loadConnections()
  connReady.value = true
  connRefreshTimer = window.setInterval(() => {
    loadConnections()
  }, 5000)
})

onUnmounted(() => {
  clearConnSearchDebounce()
  if (connRefreshTimer !== null) {
    window.clearInterval(connRefreshTimer)
    connRefreshTimer = null
  }
})

// ── helpers ───────────────────────────────────────────────────────────────────
function goBack() {
  router.push({name: 'tunnels'})
}

function openClient(sessionId: string) {
  if (!sessionId) return
  router.push({name: 'client-detail', params: {sessionId}})
}

function requestDelete() {
  confirmDelete.value = true
}

function cancelDelete() {
  confirmDelete.value = false
}

async function confirmAndDelete() {
  if (deleting.value) return
  deleting.value = true
  try {
    await kickProxy(name.value)
    showToast('info', t('tunnels.deleteSuccess', {name: name.value}))
    router.push({name: 'tunnels'})
  } catch (e) {
    const msg = e instanceof Error ? e.message : String(e)
    showToast('error', msg || t('tunnels.deleteFailed', {name: name.value}))
    confirmDelete.value = false
  } finally {
    deleting.value = false
  }
}
</script>

<template>
  <div class="detail">
    <button class="back" type="button" @click="goBack">← {{ t('tunnels.back') }}</button>

    <section class="summary card">
      <div class="summary-head">
        <div class="head-left">
          <div class="avatar" aria-hidden="true">
            <AppIcon name="link"/>
          </div>
          <div class="head-body">
            <div class="title-row">
              <h2 class="name">{{ tunnel?.name || name }}</h2>
              <span class="type-badge">{{ (tunnel?.type || '—').toUpperCase() }}</span>
              <StatusBadge
                  size="sm"
                  :status="isOnline(tunnel?.status) ? 'online' : 'offline'"
                  :label="statusLabel(tunnel?.status)"
              />
            </div>
            <div class="meta">
              <button
                  v-if="tunnel?.sessionId"
                  type="button"
                  class="meta-client"
                  :title="t('tunnels.openClient')"
                  :aria-label="t('tunnels.openClient')"
                  @click="openClient(tunnel.sessionId)"
              >
                <AppIcon name="monitor"/>
                <code>{{ tunnel.sessionId }}</code>
              </button>
              <span v-else class="meta-client is-empty">
                <AppIcon name="monitor"/>
                <code>—</code>
              </span>
              <template v-if="tunnel?.lastStartTime">
                <span class="meta-sep" aria-hidden="true">·</span>
                <span class="meta-text">
                  {{ t('tunnels.lastStarted', {time: tunnel.lastStartTime}) }}
                </span>
              </template>
            </div>
          </div>
        </div>

        <!-- delete action -->
        <div class="head-right">
          <Transition name="confirm">
            <div v-if="confirmDelete" class="confirm-bar">
              <span class="confirm-label">{{ t('tunnels.deleteConfirm') }}</span>
              <button
                  type="button"
                  class="confirm-ok"
                  :disabled="deleting"
                  @click="confirmAndDelete"
              >
                {{ deleting ? t('tunnels.deleting') : t('tunnels.deleteOk') }}
              </button>
              <button
                  type="button"
                  class="confirm-cancel"
                  :disabled="deleting"
                  @click="cancelDelete"
              >
                {{ t('tunnels.deleteCancel') }}
              </button>
            </div>
          </Transition>
          <button
              v-if="!confirmDelete"
              type="button"
              class="delete-btn"
              :title="t('tunnels.delete')"
              :aria-label="t('tunnels.delete')"
              @click="requestDelete"
          >
            <AppIcon name="kick"/>
          </button>
        </div>
      </div>

      <div class="metrics" role="list">
        <div class="metric" role="listitem">
          <em>{{ isHttpTunnelType(tunnel?.type) ? t('tunnels.domain') : t('tunnels.port') }}</em>
          <div class="metric-value mono">
            {{ formatTunnelEndpoint(tunnel?.type, tunnel?.remoteAddr) }}
          </div>
        </div>
        <div class="metric" role="listitem">
          <em>{{ t('tunnels.localAddr') }}</em>
          <div class="metric-value mono">{{ tunnel?.localAddr || '—' }}</div>
        </div>
        <div class="metric" role="listitem">
          <em>{{ t('tunnels.activeConnections') }}</em>
          <div class="metric-value">{{ tunnel?.activeConnections ?? 0 }}</div>
        </div>
        <div class="metric" role="listitem">
          <em>{{ t('tunnels.traffic') }}</em>
          <div class="metric-value">
            <TrafficIO
                layout="inline"
                :traffic-in="tunnel?.todayTrafficIn"
                :traffic-out="tunnel?.todayTrafficOut"
            />
          </div>
        </div>
      </div>
    </section>

    <SectionCard :title="t('traffic.history')">
      <template #extra>
        <div class="chart-toolbar">
          <div class="range-toggle" role="group" :aria-label="t('traffic.chartType')">
            <button
                type="button"
                class="range-btn"
                :class="{ active: chartVariant === 'line' }"
                @click="chartVariant = 'line'"
            >
              {{ t('traffic.chartLine') }}
            </button>
            <button
                type="button"
                class="range-btn"
                :class="{ active: chartVariant === 'bar' }"
                @click="chartVariant = 'bar'"
            >
              {{ t('traffic.chartBar') }}
            </button>
          </div>
          <div class="range-toggle" role="group" :aria-label="t('traffic.range')">
            <button
                type="button"
                class="range-btn"
                :class="{ active: trafficRange === '24h' }"
                @click="trafficRange = '24h'"
            >
              {{ t('traffic.range24h') }}
            </button>
            <button
                type="button"
                class="range-btn"
                :class="{ active: trafficRange === '7d' }"
                @click="trafficRange = '7d'"
            >
              {{ t('traffic.range7d') }}
            </button>
          </div>
        </div>
      </template>
      <TrafficChart
          :tunnel-name="name"
          :range="trafficRange"
          :variant="chartVariant"
          :refresh-ms="5000"
      />
    </SectionCard>

    <!-- ── connections panel ── -->
    <section class="conn-panel card">
      <div class="conn-header">
        <div class="conn-title">
          <h3>{{ t('tunnels.connectionsTitle') }}</h3>
          <span class="conn-count">{{ connTotal }}</span>
        </div>
        <label class="conn-search">
          <span class="conn-search-icon" aria-hidden="true">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none"
                 stroke="currentColor" stroke-width="2.2"
                 stroke-linecap="round" stroke-linejoin="round">
              <circle cx="11" cy="11" r="8"/>
              <path d="M21 21l-4.35-4.35"/>
            </svg>
          </span>
          <input
              v-model="connSearch"
              type="search"
              class="conn-search-input"
              :placeholder="t('tunnels.connectionsSearchPlaceholder')"
              :aria-label="t('tunnels.connectionsSearchPlaceholder')"
          />
          <button
              v-if="connSearch"
              type="button"
              class="conn-search-clear"
              :aria-label="t('tunnels.connectionsSearchClear')"
              @click="connSearch = ''"
          >
            <svg width="12" height="12" viewBox="0 0 24 24" fill="none"
                 stroke="currentColor" stroke-width="2.5"
                 stroke-linecap="round" stroke-linejoin="round">
              <path d="M18 6L6 18M6 6l12 12"/>
            </svg>
          </button>
        </label>
      </div>

      <div v-if="connLoading && !connections.length" class="conn-empty">
        {{ t('traffic.loading') }}
      </div>
      <div v-else-if="!connections.length" class="conn-empty">
        {{
          connSearch.trim()
            ? t('tunnels.connectionsSearchEmpty', {q: connSearch.trim()})
            : t('tunnels.connectionsEmpty')
        }}
      </div>

      <div v-else class="conn-list">
        <div
            v-for="conn in connections"
            :key="conn.id"
            class="conn-row"
        >
          <span class="conn-remote mono">
            <em>{{ t('tunnels.connectionRemoteAddr') }}</em>
            {{ conn.remoteAddr || '—' }}
          </span>
          <span v-if="conn.localAddr" class="conn-local mono">
            <em>{{ t('tunnels.connectionLocalAddr') }}</em>
            {{ conn.localAddr }}
          </span>
          <span v-if="conn.connectedAt" class="conn-time">
            <em>{{ t('tunnels.connectionConnectedAt') }}</em>
            {{ conn.connectedAt }}
          </span>
          <span v-if="conn.trafficIn != null || conn.trafficOut != null" class="conn-traffic">
            <TrafficIO
                layout="inline"
                :traffic-in="conn.trafficIn"
                :traffic-out="conn.trafficOut"
            />
          </span>
        </div>
      </div>

      <PaginationBar
          v-if="connTotal > 0"
          v-model:page="connPage"
          v-model:page-size="connPageSize"
          :total="connTotal"
      />
    </section>
  </div>
</template>

<style scoped>
.detail {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.back {
  align-self: flex-start;
  border: 0;
  background: transparent;
  color: var(--muted);
  cursor: pointer;
  padding: 0;
  font: inherit;
}

.back:hover {
  color: inherit;
}

.summary {
  display: flex;
  flex-direction: column;
  gap: 1rem;
  padding: 1.1rem 1.2rem 1.15rem;
}

.summary-head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 1rem;
}

.head-left {
  display: flex;
  align-items: center;
  gap: 0.9rem;
  min-width: 0;
}

.avatar {
  width: 2.75rem;
  height: 2.75rem;
  border-radius: var(--radius);
  display: grid;
  place-items: center;
  flex-shrink: 0;
  font-size: 1.35rem;
  color: var(--accent-text);
  background: var(--accent-soft);
  border: 1px solid color-mix(in srgb, var(--accent) 22%, transparent);
}

.head-body {
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 0.55rem;
}

.title-row {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 0.5rem;
}

.name {
  margin: 0;
  font-size: 1.25rem;
  font-weight: 700;
  letter-spacing: -0.02em;
  line-height: 1.2;
}

.type-badge {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-height: 1.55rem;
  padding: 0.18rem 0.65rem;
  border-radius: var(--radius-pill);
  font-size: 0.72rem;
  font-weight: 650;
  line-height: 1.2;
  color: var(--muted);
  background: color-mix(in srgb, var(--muted) 12%, transparent);
  border: 1px solid color-mix(in srgb, var(--muted) 18%, transparent);
}

.meta {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 0.35rem 0.55rem;
  color: var(--muted);
  font-size: 0.82rem;
}

.meta-client {
  display: inline-flex;
  align-items: center;
  gap: 0.35rem;
  min-width: 0;
  max-width: 100%;
  padding: 0;
  border: 0;
  background: transparent;
  color: var(--muted);
  font: inherit;
  cursor: pointer;
}

.meta-client:not(.is-empty):hover {
  color: var(--accent-text);
}

.meta-client:not(.is-empty):hover code {
  color: var(--accent-text);
}

.meta-client :deep(svg) {
  width: 0.95rem;
  height: 0.95rem;
  flex-shrink: 0;
}

.meta-client.is-empty {
  cursor: default;
}

.meta-sep {
  color: var(--muted);
  opacity: 0.7;
  user-select: none;
}

.meta-text {
  color: var(--muted);
}

.meta code,
.mono {
  font-family: 'IBM Plex Mono', ui-monospace, monospace;
  font-size: 0.9em;
  color: var(--text);
  word-break: break-all;
}

/* ── delete / confirm ── */
.head-right {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  flex-shrink: 0;
}

.delete-btn {
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
  transition: border-color 0.15s ease, background 0.15s ease;
}

.delete-btn:hover {
  border-color: var(--danger, #ef4444);
  background: color-mix(in srgb, var(--danger, #ef4444) 8%, transparent);
}

.confirm-bar {
  display: inline-flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.3rem 0.55rem;
  border-radius: var(--radius);
  background: color-mix(in srgb, var(--danger, #ef4444) 8%, var(--panel));
  border: 1px solid color-mix(in srgb, var(--danger, #ef4444) 30%, transparent);
}

.confirm-label {
  font-size: 0.8rem;
  color: var(--text);
  white-space: nowrap;
}

.confirm-ok {
  font: inherit;
  font-size: 0.78rem;
  font-weight: 600;
  padding: 0.22rem 0.7rem;
  border-radius: var(--radius);
  border: 1px solid color-mix(in srgb, var(--danger, #ef4444) 55%, transparent);
  background: color-mix(in srgb, var(--danger, #ef4444) 12%, transparent);
  color: var(--danger, #ef4444);
  cursor: pointer;
  transition: background 0.15s ease;
}

.confirm-ok:hover:not(:disabled) {
  background: color-mix(in srgb, var(--danger, #ef4444) 20%, transparent);
}

.confirm-ok:disabled {
  opacity: 0.5;
  cursor: wait;
}

.confirm-cancel {
  font: inherit;
  font-size: 0.78rem;
  font-weight: 500;
  padding: 0.22rem 0.7rem;
  border-radius: var(--radius);
  border: 1px solid var(--line);
  background: transparent;
  color: var(--muted);
  cursor: pointer;
  transition: color 0.15s ease, border-color 0.15s ease;
}

.confirm-cancel:hover:not(:disabled) {
  color: var(--text);
  border-color: var(--line-strong);
}

.confirm-cancel:disabled {
  opacity: 0.5;
  cursor: wait;
}

/* transition */
.confirm-enter-active,
.confirm-leave-active {
  transition: opacity 0.15s ease, transform 0.15s ease;
}

.confirm-enter-from,
.confirm-leave-to {
  opacity: 0;
  transform: translateX(6px);
}

/* ── metrics ── */
.metrics {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 0.75rem;
  padding-top: 0.95rem;
  border-top: 1px solid var(--line);
}

.metric {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 0.4rem;
  min-width: 0;
}

.metric em {
  display: block;
  font-style: normal;
  font-size: 0.75rem;
  font-weight: 600;
  line-height: 1.2;
  color: var(--muted);
  letter-spacing: 0.01em;
}

.metric-value {
  display: flex;
  align-items: center;
  min-height: 1.5rem;
  font-size: 1.05rem;
  font-weight: 700;
  letter-spacing: -0.02em;
  line-height: 1.2;
  word-break: break-all;
}

.metric-value :deep(.traffic-io.inline) {
  gap: 0.35rem;
}

.metric-value :deep(.row) {
  font-size: 1.05rem;
  font-weight: 700;
  line-height: 1.2;
}

.metric-value :deep(.sep) {
  font-size: 1.05rem;
  font-weight: 600;
  line-height: 1.2;
}

.metric-value :deep(.arrow) {
  width: 1rem;
  height: 1rem;
}

/* ── chart toolbar ── */
.chart-toolbar {
  display: inline-flex;
  flex-wrap: wrap;
  gap: 0.5rem;
  justify-content: flex-end;
}

.range-toggle {
  display: inline-flex;
  padding: 2px;
  border-radius: var(--radius-pill);
  background: color-mix(in srgb, var(--muted) 10%, transparent);
  border: 1px solid var(--line);
}

.range-btn {
  border: 0;
  background: transparent;
  color: var(--muted);
  font: inherit;
  font-size: 0.72rem;
  font-weight: 600;
  padding: 0.22rem 0.65rem;
  border-radius: var(--radius-pill);
  cursor: pointer;
}

.range-btn.active {
  color: var(--accent-text);
  background: var(--panel);
  box-shadow: var(--shadow);
}

.range-btn:hover:not(.active) {
  color: var(--text);
}

/* ── connections panel ── */
.conn-panel {
  display: flex;
  flex-direction: column;
  gap: 0;
  padding: 0;
  overflow: hidden;
}

.conn-header {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  justify-content: space-between;
  gap: 0.75rem;
  padding: 0.95rem 1.15rem;
  border-bottom: 1px solid var(--line);
}

.conn-title {
  display: inline-flex;
  align-items: center;
  gap: 0.5rem;
}

.conn-title h3 {
  margin: 0;
  font-size: 0.95rem;
  font-weight: 650;
}

.conn-count {
  min-width: 1.35rem;
  padding: 0.1rem 0.45rem;
  border-radius: var(--radius);
  text-align: center;
  font-size: 0.72rem;
  font-weight: 650;
  font-variant-numeric: tabular-nums;
  color: var(--muted);
  background: color-mix(in srgb, var(--muted) 12%, transparent);
}

.conn-search {
  display: inline-flex;
  align-items: center;
  gap: 0.4rem;
  min-width: min(100%, 14rem);
  padding: 0.35rem 0.7rem;
  border-radius: var(--radius);
  border: 1px solid var(--line);
  background: color-mix(in srgb, var(--muted) 6%, transparent);
}

.conn-search-icon {
  width: 0.9rem;
  height: 0.9rem;
  display: inline-grid;
  place-items: center;
  line-height: 0;
  color: var(--muted);
  flex-shrink: 0;
}

.conn-search-icon :deep(svg) {
  width: 100%;
  height: 100%;
  display: block;
}

.conn-search input {
  flex: 1;
  min-width: 0;
  border: 0;
  outline: none;
  background: transparent;
  color: var(--text);
  font: inherit;
  font-size: 0.82rem;
}

.conn-search input::placeholder {
  color: var(--muted);
}

.conn-search-clear {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 1rem;
  height: 1rem;
  border-radius: 50%;
  border: 0;
  background: color-mix(in srgb, var(--muted) 18%, transparent);
  color: var(--muted);
  cursor: pointer;
  padding: 0;
  flex-shrink: 0;
}

.conn-search-clear:hover {
  background: color-mix(in srgb, var(--muted) 32%, transparent);
  color: var(--text);
}

.conn-empty {
  padding: 2.25rem 1rem;
  text-align: center;
  color: var(--muted);
}

.conn-list {
  display: flex;
  flex-direction: column;
}

.conn-row {
  display: flex;
  flex-wrap: wrap;
  align-items: baseline;
  gap: 0.35rem 1.25rem;
  padding: 0.75rem 1.15rem;
  border-bottom: 1px solid var(--line);
  font-size: 0.82rem;
}

.conn-row:last-child {
  border-bottom: 0;
}

.conn-remote,
.conn-local {
  font-family: 'IBM Plex Mono', ui-monospace, monospace;
  font-size: 0.78rem;
  color: var(--text);
  word-break: break-all;
}

.conn-time {
  color: var(--muted);
  font-size: 0.78rem;
}

.conn-row em {
  font-style: normal;
  color: var(--muted);
  font-size: 0.72rem;
  font-weight: 600;
  margin-right: 0.25rem;
}

.conn-panel :deep(.pagination-bar) {
  padding: 0 1rem 0.9rem;
  border-top: 0;
}

@media (max-width: 960px) {
  .metrics {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
}

@media (max-width: 520px) {
  .metrics {
    grid-template-columns: 1fr;
  }

  .confirm-bar {
    flex-wrap: wrap;
  }

  .confirm-label {
    width: 100%;
  }

  .conn-row {
    flex-direction: column;
    gap: 0.2rem;
  }
}
</style>
