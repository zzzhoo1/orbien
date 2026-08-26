<script setup lang="ts">
import {computed} from 'vue'
import {formatFileSize} from '@/utils/format'
import {useLocale} from '@/composables/useLocale'
import arrowUpIcon from '@/assets/icon/arrow-up.svg?raw'
import arrowDownIcon from '@/assets/icon/arrow-down.svg?raw'

const props = withDefaults(
    defineProps<{
      trafficIn?: number | null
      trafficOut?: number | null
      variant?: 'plain' | 'chip'
      layout?: 'stack' | 'inline'
    }>(),
    {
      trafficIn: 0,
      trafficOut: 0,
      variant: 'plain',
      layout: 'stack',
    },
)

const {t} = useLocale()

const inbound = computed(() => Number(props.trafficIn ?? 0) || 0)
const outbound = computed(() => Number(props.trafficOut ?? 0) || 0)
</script>

<template>
  <div
      class="traffic-io"
      :class="[variant, layout]"
      :title="`${t('traffic.out')}: ${formatFileSize(outbound)} · ${t('traffic.in')}: ${formatFileSize(inbound)}`"
  >
    <div class="row out">
      <span class="arrow" aria-hidden="true" v-html="arrowUpIcon"/>
      <span class="val">{{ formatFileSize(outbound) }}</span>
    </div>
    <span v-if="layout === 'inline'" class="sep" aria-hidden="true">/</span>
    <div class="row in">
      <span class="arrow" aria-hidden="true" v-html="arrowDownIcon"/>
      <span class="val">{{ formatFileSize(inbound) }}</span>
    </div>
  </div>
</template>

<style scoped>
.traffic-io {
  display: inline-flex;
  flex-direction: column;
  justify-content: center;
  gap: 0.28rem;
  line-height: 1.15;
  min-width: 4.5rem;
}

.traffic-io.inline {
  flex-direction: row;
  align-items: center;
  gap: 0.4rem;
  min-width: 0;
  line-height: 1.2;
}

.traffic-io.inline .row {
  line-height: 1.2;
}

.traffic-io.inline .val {
  font-weight: inherit;
}

.traffic-io.chip {
  padding: 0.28rem 0.55rem;
  border-radius: var(--radius);
  background: color-mix(in srgb, var(--muted) 8%, transparent);
  border: 1px solid color-mix(in srgb, var(--line) 80%, transparent);
}

.row {
  display: flex;
  align-items: center;
  gap: 0.28rem;
  font-variant-numeric: tabular-nums;
  font-size: 0.8rem;
  font-weight: 600;
  white-space: nowrap;
}

.sep {
  color: var(--muted);
  font-size: 0.85rem;
  font-weight: 500;
  line-height: 1;
  user-select: none;
}

.arrow {
  width: 0.72rem;
  height: 0.72rem;
  flex-shrink: 0;
  display: inline-grid;
  place-items: center;
  line-height: 0;
  color: inherit;
}

.arrow :deep(svg) {
  width: 100%;
  height: 100%;
  display: block;
  fill: currentColor;
  stroke: none;
}

.row.out {
  color: #14b8a6;
}

.row.in {
  color: #3b82f6;
}

.val {
  font-weight: 500;
  letter-spacing: 0.01em;
}
</style>
