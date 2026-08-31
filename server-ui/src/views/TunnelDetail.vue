<script setup lang="ts">
import {computed, ref} from 'vue'
import {useRoute, useRouter} from 'vue-router'
import AppIcon from '@/components/AppIcon.vue'
import SectionCard from '@/components/SectionCard.vue'
import StatusBadge from '@/components/StatusBadge.vue'
import TrafficChart from '@/components/TrafficChart.vue'
import TrafficIO from '@/components/TrafficIO.vue'
import type {TrafficRange} from '@/api'
import {kickProxy} from '@/api'
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

// ── delete state ────────────────────────────────────────────────────────────────
const confirmDelete = ref(false)
const deleting = ref(false)

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
}
</style>
