<script setup lang="ts">
import {computed} from 'vue'
import {formatFileSize} from '@/utils/format'
import {useLocale} from '@/composables/useLocale'
import downloadIcon from '@/assets/icon/download.svg?raw'
import uploadIcon from '@/assets/icon/upload.svg?raw'

const props = withDefaults(
    defineProps<{
      trafficIn?: number | null
      trafficOut?: number | null
    }>(),
    {
      trafficIn: 0,
      trafficOut: 0,
    },
)

const {t} = useLocale()

const inbound = computed(() => Number(props.trafficIn ?? 0) || 0)
const outbound = computed(() => Number(props.trafficOut ?? 0) || 0)
const total = computed(() => inbound.value + outbound.value)

const inShare = computed(() => {
  if (total.value <= 0) return 50
  return Math.round((inbound.value / total.value) * 100)
})
const outShare = computed(() => 100 - inShare.value)
</script>

<template>
  <div class="traffic-summary">
    <div class="traffic-total">
      <div class="total-label">{{ t('traffic.total') }}</div>
      <div class="total-value">{{ formatFileSize(total) }}</div>
    </div>

    <div class="traffic-split">
      <div class="traffic-item in">
        <div class="traffic-icon is-asset" aria-hidden="true" v-html="downloadIcon"/>
        <div class="traffic-meta">
          <div class="traffic-label">{{ t('traffic.in') }}</div>
          <div class="traffic-value">{{ formatFileSize(inbound) }}</div>
        </div>
      </div>

      <div class="traffic-divider" aria-hidden="true"/>

      <div class="traffic-item out">
        <div class="traffic-icon is-asset" aria-hidden="true" v-html="uploadIcon"/>
        <div class="traffic-meta">
          <div class="traffic-label">{{ t('traffic.out') }}</div>
          <div class="traffic-value">{{ formatFileSize(outbound) }}</div>
        </div>
      </div>
    </div>

    <div class="traffic-bar" role="img" :aria-label="t('traffic.total')">
      <span class="bar-in" :style="{ width: `${inShare}%` }"/>
      <span class="bar-out" :style="{ width: `${outShare}%` }"/>
    </div>
    <div class="traffic-bar-legend">
      <span class="leg in">{{ t('traffic.in') }} {{ inShare }}%</span>
      <span class="leg out">{{ t('traffic.out') }} {{ outShare }}%</span>
    </div>
  </div>
</template>

<style scoped>
.traffic-summary {
  display: flex;
  flex-direction: column;
  gap: 1.15rem;
  min-height: 11rem;
}

.traffic-total {
  display: flex;
  flex-direction: column;
  gap: 0.2rem;
}

.total-label {
  color: var(--muted);
  font-size: 0.82rem;
}

.total-value {
  font-size: 1.85rem;
  font-weight: 700;
  letter-spacing: -0.02em;
  font-variant-numeric: tabular-nums;
  color: var(--text);
  line-height: 1.15;
}

.traffic-split {
  display: grid;
  grid-template-columns: 1fr auto 1fr;
  align-items: center;
  gap: 0.75rem;
}

.traffic-item {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  min-width: 0;
}

.traffic-icon {
  width: 2.6rem;
  height: 2.6rem;
  border-radius: var(--radius);
  display: grid;
  place-items: center;
  flex-shrink: 0;
}

.traffic-icon.is-asset {
  line-height: 0;
}

.traffic-icon.is-asset :deep(svg) {
  width: 1.15rem;
  height: 1.15rem;
  display: block;
  fill: currentColor;
  stroke: none;
}

.traffic-item.in .traffic-icon {
  color: #3b82f6;
  background: rgba(59, 130, 246, 0.14);
}

.traffic-item.out .traffic-icon {
  color: #14b8a6;
  background: rgba(20, 184, 166, 0.14);
}

.traffic-meta {
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 0.15rem;
}

.traffic-label {
  color: var(--muted);
  font-size: 0.8rem;
}

.traffic-value {
  font-size: 1.2rem;
  font-weight: 650;
  font-variant-numeric: tabular-nums;
  color: var(--text);
  word-break: break-all;
}

.traffic-divider {
  width: 1px;
  height: 2.75rem;
  background: var(--line);
}

.traffic-bar {
  display: flex;
  height: 0.45rem;
  border-radius: var(--radius-pill);
  overflow: hidden;
  background: var(--line);
}

.bar-in {
  background: #3b82f6;
  transition: width 0.35s ease;
}

.bar-out {
  background: #14b8a6;
  transition: width 0.35s ease;
}

.traffic-bar-legend {
  display: flex;
  justify-content: space-between;
  gap: 1rem;
  font-size: 0.75rem;
  color: var(--muted);
}

.leg.in::before,
.leg.out::before {
  content: '';
  display: inline-block;
  width: 0.5rem;
  height: 0.5rem;
  border-radius: var(--radius);
  margin-right: 0.35rem;
  vertical-align: middle;
}

.leg.in::before {
  background: #3b82f6;
}

.leg.out::before {
  background: #14b8a6;
}

@media (max-width: 560px) {
  .traffic-split {
    grid-template-columns: 1fr;
    gap: 0.85rem;
  }

  .traffic-divider {
    width: 100%;
    height: 1px;
  }
}
</style>
