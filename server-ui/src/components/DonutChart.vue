<script setup lang="ts">
import {computed} from 'vue'
import {useLocale} from '@/composables/useLocale'

export interface ChartSlice {
  key: string
  label: string
  value: number
  color: string
}

const props = defineProps<{
  slices: ChartSlice[]

  size?: number
}>()

const {t} = useLocale()

const size = computed(() => props.size ?? 200)
const cx = computed(() => size.value / 2)
const cy = computed(() => size.value / 2)
const r = computed(() => size.value * 0.36)
const stroke = computed(() => size.value * 0.14)

const total = computed(() => props.slices.reduce((s, x) => s + Math.max(0, x.value), 0))

const arcs = computed(() => {
  const sum = total.value
  if (sum <= 0) return []
  let angle = -Math.PI / 2
  return props.slices
      .filter((s) => s.value > 0)
      .map((s) => {
        const portion = s.value / sum
        const delta = portion * Math.PI * 2
        const start = angle
        const end = angle + delta
        angle = end
        return {
          ...s,
          portion,
          path: donutArc(cx.value, cy.value, r.value, start, end),
        }
      })
})

function donutArc(x: number, y: number, radius: number, start: number, end: number) {

  if (end - start >= Math.PI * 2 - 1e-6) {
    const d = radius
    return `M ${x} ${y - d} A ${d} ${d} 0 1 1 ${x - 0.01} ${y - d}`
  }
  const sx = x + radius * Math.cos(start)
  const sy = y + radius * Math.sin(start)
  const ex = x + radius * Math.cos(end)
  const ey = y + radius * Math.sin(end)
  const large = end - start > Math.PI ? 1 : 0
  return `M ${sx} ${sy} A ${radius} ${radius} 0 ${large} 1 ${ex} ${ey}`
}
</script>

<template>
  <div class="donut">
    <svg
        class="donut-svg"
        :viewBox="`0 0 ${size} ${size}`"
        role="img"
        :aria-label="t('monitor.tunnelTypes')"
    >
      <circle
          v-if="!arcs.length"
          :cx="cx"
          :cy="cy"
          :r="r"
          fill="none"
          :stroke-width="stroke"
          class="donut-empty-ring"
      />
      <path
          v-for="arc in arcs"
          :key="arc.key"
          :d="arc.path"
          fill="none"
          :stroke="arc.color"
          :stroke-width="stroke"
          stroke-linecap="butt"
      />
      <text
          :x="cx"
          :y="cy - 4"
          text-anchor="middle"
          class="donut-total"
      >
        {{ total }}
      </text>
      <text
          :x="cx"
          :y="cy + 14"
          text-anchor="middle"
          class="donut-total-label"
      >
        {{ t('monitor.chartTotal') }}
      </text>
    </svg>

    <ul v-if="slices.length" class="donut-legend">
      <li v-for="s in slices" :key="s.key">
        <span class="swatch" :style="{ background: s.color }"/>
        <span class="name">{{ s.label }}</span>
        <span class="count">{{ s.value }}</span>
      </li>
    </ul>
    <p v-else class="donut-empty">{{ t('common.notConfigured') }}</p>
  </div>
</template>

<style scoped>
.donut {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 1rem;
}

.donut-svg {
  width: min(100%, 220px);
  height: auto;
}

.donut-empty-ring {
  stroke: var(--line);
}

.donut-total {
  fill: var(--text);
  font-size: 22px;
  font-weight: 700;
  font-family: inherit;
}

.donut-total-label {
  fill: var(--muted);
  font-size: 11px;
  font-family: inherit;
}

.donut-legend {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-wrap: wrap;
  justify-content: center;
  gap: 0.55rem 1rem;
  width: 100%;
}

.donut-legend li {
  display: inline-flex;
  align-items: center;
  gap: 0.4rem;
  font-size: 0.82rem;
  color: var(--text-secondary);
}

.swatch {
  width: 0.65rem;
  height: 0.65rem;
  border-radius: var(--radius);
  flex-shrink: 0;
}

.name {
  text-transform: uppercase;
  letter-spacing: 0.02em;
}

.count {
  color: var(--muted);
  font-variant-numeric: tabular-nums;
}

.donut-empty {
  margin: 0;
  color: var(--muted);
  font-size: 0.9rem;
}
</style>
