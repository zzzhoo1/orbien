<script setup lang="ts">
import {computed, ref} from 'vue'
import {useRoute, useRouter} from 'vue-router'
import AppIcon from '@/components/AppIcon.vue'
import SectionCard from '@/components/SectionCard.vue'
import TrafficChart from '@/components/TrafficChart.vue'
import TrafficIO from '@/components/TrafficIO.vue'
import type {TrafficRange} from '@/api/client'
import {useDashboardStore} from '@/stores/dashboard'
import {useLocale} from '@/composables/useLocale'
import {usePresence} from '@/composables/usePresence'
import {formatTunnelEndpoint, isHttpTunnelType} from '@/utils/format'

const route = useRoute()
const router = useRouter()
const store = useDashboardStore()
const {t} = useLocale()
const {isOnline, statusLabel} = usePresence()
const trafficRange = ref<TrafficRange>('24h')
const chartVariant = ref<'bar' | 'line'>('bar')
const name = computed(() => String(route.params.name || ''))
const tunnel = computed(() => store.tunnels.find((t) => t.name === name.value) || null)

function goBack() {
  router.push({name: 'tunnels'})
}

function openClient(sessionId: string) {
  if (!sessionId) return
  router.push({name: 'client-detail', params: {sessionId}})
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
              <span class="status-badge" :class="{ online: isOnline(tunnel?.status) }">
                {{ statusLabel(tunnel?.status) }}
              </span>
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

.type-badge,
.status-badge {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-height: 1.55rem;
  padding: 0.18rem 0.65rem;
  border-radius: var(--radius-pill);
  font-size: 0.72rem;
  font-weight: 650;
  line-height: 1.2;
}

.type-badge {
  color: var(--muted);
  background: color-mix(in srgb, var(--muted) 12%, transparent);
  border: 1px solid color-mix(in srgb, var(--muted) 18%, transparent);
}

.status-badge {
  color: var(--muted);
  background: color-mix(in srgb, var(--muted) 12%, transparent);
  border: 1px solid color-mix(in srgb, var(--muted) 18%, transparent);
}

.status-badge.online {
  color: var(--status-ok);
  background: var(--status-ok-soft);
  border-color: color-mix(in srgb, var(--status-ok) 22%, transparent);
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
}
</style>
