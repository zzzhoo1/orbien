<script setup lang="ts">
import {computed, onMounted, onUnmounted, ref, watch} from 'vue'
import {fetchTunnelTraffic, fetchSystemTraffic, type TrafficRange} from '@/api/client'
import {formatFileSize} from '@/utils/format'
import {useLocale} from '@/composables/useLocale'

const props = withDefaults(
    defineProps<{
      tunnelName?: string
      range?: TrafficRange
      variant?: 'bar' | 'line'
      refreshMs?: number
    }>(),
    {
      tunnelName: '',
      range: '7d',
      variant: 'bar',
      refreshMs: 0,
    },
)

const {t} = useLocale()
const loading = ref(false)
const error = ref<string | null>(null)
const points = ref<Array<{ date: string; in: number; out: number }>>([])
const granularity = ref<'day' | 'hour' | string>('day')
const hoverIndex = ref<number | null>(null)
let loadSeq = 0

const maxVal = computed(() => {
  const m = Math.max(0, ...points.value.flatMap((p) => [p.in, p.out]))
  return Math.max(m, 100)
})

const dense = computed(() => props.range === '24h')
const isLine = computed(() => props.variant === 'line')

const W = 720
const H = 200
const PAD = {top: 10, right: 20, bottom: 22, left: 20}

const plotW = computed(() => W - PAD.left - PAD.right)
const plotH = computed(() => H - PAD.top - PAD.bottom)
const axisY = computed(() => PAD.top + plotH.value)
const labelY = computed(() => axisY.value + 13)

function xAt(i: number, n: number) {
  if (n <= 1) return PAD.left + plotW.value / 2
  return PAD.left + (i / (n - 1)) * plotW.value
}

function yAt(value: number) {
  const ratio = Math.min(1, Math.max(0, value / maxVal.value))
  return PAD.top + plotH.value * (1 - ratio)
}

const markers = computed(() => {
  const n = points.value.length
  return points.value.map((p, i) => ({
    i,
    x: xAt(i, n),
    yIn: yAt(p.in),
    yOut: yAt(p.out),
    date: p.date,
  }))
})

const bars = computed(() => {
  const n = points.value.length
  if (!n) return []
  const spacing = n <= 1 ? plotW.value : plotW.value / (n - 1)
  const pairW = Math.min(spacing * (dense.value ? 0.55 : 0.7), dense.value ? 12 : 26)
  const gap = Math.max(1, pairW * 0.14)
  const barW = Math.max(1.5, (pairW - gap) / 2)
  const base = axisY.value

  return points.value.map((p, i) => {
    const cx = xAt(i, n)
    const yIn = yAt(p.in)
    const yOut = yAt(p.out)
    const hIn = p.in > 0 ? Math.max(base - yIn, 2) : 0
    const hOut = p.out > 0 ? Math.max(base - yOut, 2) : 0
    return {
      i,
      key: `${props.range}:${p.date}:${i}`,
      xIn: cx - gap / 2 - barW,
      xOut: cx + gap / 2,
      yIn: base - hIn,
      yOut: base - hOut,
      hIn,
      hOut,
      w: barW,
      date: p.date,
    }
  })
})

const lineIn = computed(() => {
  const n = points.value.length
  return points.value
      .map((p, i) => `${i === 0 ? 'M' : 'L'}${xAt(i, n).toFixed(2)},${yAt(p.in).toFixed(2)}`)
      .join(' ')
})

const lineOut = computed(() => {
  const n = points.value.length
  return points.value
      .map((p, i) => `${i === 0 ? 'M' : 'L'}${xAt(i, n).toFixed(2)},${yAt(p.out).toFixed(2)}`)
      .join(' ')
})

const areaIn = computed(() => {
  const n = points.value.length
  if (!n) return ''
  const baseY = axisY.value
  const head = `M${xAt(0, n).toFixed(2)},${baseY.toFixed(2)}`
  const mid = points.value
      .map((p, i) => `L${xAt(i, n).toFixed(2)},${yAt(p.in).toFixed(2)}`)
      .join('')
  const tail = `L${xAt(n - 1, n).toFixed(2)},${baseY.toFixed(2)} Z`
  return `${head}${mid}${tail}`
})

const areaOut = computed(() => {
  const n = points.value.length
  if (!n) return ''
  const baseY = axisY.value
  const head = `M${xAt(0, n).toFixed(2)},${baseY.toFixed(2)}`
  const mid = points.value
      .map((p, i) => `L${xAt(i, n).toFixed(2)},${yAt(p.out).toFixed(2)}`)
      .join('')
  const tail = `L${xAt(n - 1, n).toFixed(2)},${baseY.toFixed(2)} Z`
  return `${head}${mid}${tail}`
})

const xLabels = computed(() => {
  const n = points.value.length
  if (!n) return []
  return points.value.map((p, i) => {
    const atStart = i === 0
    const atEnd = i === n - 1
    return {
      i,
      x: xAt(i, n),
      text: dense.value ? shortHourLabel(p.date) : p.date,
      anchor: atStart ? 'start' : atEnd ? 'end' : 'middle',
    }
  })
})

function shortHourLabel(date: string) {
  const m = date.match(/^(\d{1,2})/)
  return m ? m[1]! : date
}

const hoverPoint = computed(() => {
  if (hoverIndex.value == null) return null
  return points.value[hoverIndex.value] ?? null
})

const hoverX = computed(() => {
  if (hoverIndex.value == null) return 0
  return xAt(hoverIndex.value, points.value.length)
})

async function load() {
  const seq = ++loadSeq
  const range = props.range
  const name = props.tunnelName
  loading.value = true
  error.value = null
  try {
    const data = name
        ? await fetchTunnelTraffic(name, range)
        : await fetchSystemTraffic(range)
    if (seq !== loadSeq) return
    granularity.value = data.granularity || (range === '24h' ? 'hour' : 'day')
    points.value = (data.history || []).map((h) => ({
      date: formatLabel(h.date, granularity.value),
      in: Number(h.trafficIn) || 0,
      out: Number(h.trafficOut) || 0,
    }))
  } catch (e) {
    if (seq !== loadSeq) return
    error.value = e instanceof Error ? e.message : String(e)
    points.value = []
  } finally {
    if (seq === loadSeq) loading.value = false
  }
}

function formatLabel(date: string, gran: string) {
  if (gran === 'hour' || date.includes('T')) {
    const tPart = date.split('T')[1] || date
    return tPart.slice(0, 5)
  }
  const parts = date.split('-')
  if (parts.length !== 3) return date
  return `${Number(parts[1])}-${Number(parts[2])}`
}

function onMove(evt: MouseEvent) {
  const svg = evt.currentTarget as SVGSVGElement
  const rect = svg.getBoundingClientRect()
  const x = ((evt.clientX - rect.left) / rect.width) * W
  const n = points.value.length
  if (n <= 1) {
    hoverIndex.value = n ? 0 : null
    return
  }
  const t = (x - PAD.left) / plotW.value
  const i = Math.round(Math.min(1, Math.max(0, t)) * (n - 1))
  hoverIndex.value = i
}

function onLeave() {
  hoverIndex.value = null
}

let timer: ReturnType<typeof setInterval> | null = null

function restartTimer() {
  if (timer) clearInterval(timer)
  timer = null
  if (props.refreshMs > 0) {
    timer = setInterval(() => void load(), props.refreshMs)
  }
}

onMounted(() => {
  void load()
  restartTimer()
})

onUnmounted(() => {
  if (timer) clearInterval(timer)
  loadSeq += 1
})

watch(
    () => [props.tunnelName, props.range] as const,
    () => {
      hoverIndex.value = null
      points.value = []
      error.value = null
      void load()
      restartTimer()
    },
)

watch(
    () => props.variant,
    () => {
      hoverIndex.value = null
    },
)
</script>

<template>
  <div class="traffic" :class="{ dense, line: isLine }">
    <div v-if="loading && !points.length" class="muted">{{ t('traffic.loading') }}</div>
    <div v-else-if="error && !points.length" class="muted">{{ t('traffic.failed') }}</div>
    <div v-else-if="!points.length" class="muted">{{ t('traffic.empty') }}</div>
    <template v-else>
      <div class="plot-chart">
        <div class="y">
          <span>{{ formatFileSize(maxVal) }}</span>
          <span>{{ formatFileSize(maxVal / 2) }}</span>
          <span>0 B</span>
        </div>
        <div class="plot">
          <svg
              :viewBox="`0 0 ${W} ${H}`"
              preserveAspectRatio="none"
              role="img"
              @mousemove="onMove"
              @mouseleave="onLeave"
          >
            <defs>
              <linearGradient id="traffic-grad-in" x1="0" y1="0" x2="0" y2="1">
                <stop offset="0%" stop-color="#3b82f6" stop-opacity="0.28"/>
                <stop offset="100%" stop-color="#3b82f6" stop-opacity="0.02"/>
              </linearGradient>
              <linearGradient id="traffic-grad-out" x1="0" y1="0" x2="0" y2="1">
                <stop offset="0%" stop-color="#14b8a6" stop-opacity="0.22"/>
                <stop offset="100%" stop-color="#14b8a6" stop-opacity="0.02"/>
              </linearGradient>
            </defs>

            <line
                class="grid-line"
                :x1="PAD.left"
                :x2="W - PAD.right"
                :y1="PAD.top"
                :y2="PAD.top"
            />
            <line
                class="grid-line"
                :x1="PAD.left"
                :x2="W - PAD.right"
                :y1="PAD.top + plotH / 2"
                :y2="PAD.top + plotH / 2"
            />
            <line
                class="axis"
                :x1="PAD.left"
                :x2="W - PAD.right"
                :y1="axisY"
                :y2="axisY"
            />

            <!-- Line series -->
            <template v-if="isLine">
              <path :d="areaIn" fill="url(#traffic-grad-in)"/>
              <path :d="areaOut" fill="url(#traffic-grad-out)"/>
              <path :d="lineIn" class="stroke in"/>
              <path :d="lineOut" class="stroke out"/>

              <g class="markers">
                <circle
                    v-for="m in markers"
                    :key="`in-${m.i}`"
                    class="mark in"
                    :class="{ active: hoverIndex === m.i }"
                    :cx="m.x"
                    :cy="m.yIn"
                    :r="hoverIndex === m.i ? 4 : dense ? 2.2 : 3"
                />
                <circle
                    v-for="m in markers"
                    :key="`out-${m.i}`"
                    class="mark out"
                    :class="{ active: hoverIndex === m.i }"
                    :cx="m.x"
                    :cy="m.yOut"
                    :r="hoverIndex === m.i ? 4 : dense ? 2.2 : 3"
                />
              </g>
            </template>

            <g v-else class="bars">
              <template v-for="b in bars" :key="b.key">
                <rect
                    class="bar in"
                    :class="{ active: hoverIndex === b.i }"
                    :x="b.xIn"
                    :y="b.yIn"
                    :width="b.w"
                    :height="b.hIn"
                    rx="2"
                />
                <rect
                    class="bar out"
                    :class="{ active: hoverIndex === b.i }"
                    :x="b.xOut"
                    :y="b.yOut"
                    :width="b.w"
                    :height="b.hOut"
                    rx="2"
                />
              </template>
            </g>

            <g v-if="hoverIndex != null && hoverPoint">
              <line
                  class="crosshair"
                  :x1="hoverX"
                  :x2="hoverX"
                  :y1="PAD.top"
                  :y2="axisY"
              />
            </g>

            <g class="ticks">
              <line
                  v-for="lab in xLabels"
                  :key="`tick-${lab.i}`"
                  class="tick"
                  :x1="lab.x"
                  :x2="lab.x"
                  :y1="axisY"
                  :y2="axisY + 4"
              />
            </g>

            <text
                v-for="lab in xLabels"
                :key="`lab-${lab.i}`"
                class="x-label"
                :class="{ dense: dense }"
                :x="lab.x"
                :y="labelY"
                :text-anchor="lab.anchor"
            >
              {{ lab.text }}
            </text>
          </svg>

          <div
              v-if="hoverPoint"
              class="tooltip"
              :style="{ left: `${(hoverX / W) * 100}%` }"
          >
            <div class="tip-time">{{ hoverPoint.date }}</div>
            <div class="tip-row in">
              <i/> {{ t('traffic.in') }}
              <strong>{{ formatFileSize(hoverPoint.in) }}</strong>
            </div>
            <div class="tip-row out">
              <i/> {{ t('traffic.out') }}
              <strong>{{ formatFileSize(hoverPoint.out) }}</strong>
            </div>
          </div>
        </div>
      </div>

      <div class="legend">
        <span><i class="dot in"/> {{ t('traffic.in') }}</span>
        <span><i class="dot out"/> {{ t('traffic.out') }}</span>
      </div>
    </template>
  </div>
</template>

<style scoped>
.traffic {
  min-height: 180px;
}

.muted {
  color: var(--muted);
  font-size: 0.875rem;
  padding: 1.5rem 0;
  text-align: center;
}

.plot-chart {
  display: flex;
  gap: 0.45rem;
  height: 200px;
  align-items: stretch;
}

.y {
  display: flex;
  flex-direction: column;
  justify-content: space-between;
  width: 4.25rem;
  font-size: 0.7rem;
  color: var(--muted);
  text-align: right;
  padding: 10px 0 22px;
  box-sizing: border-box;
}

.plot {
  position: relative;
  flex: 1;
  min-width: 0;
  height: 100%;
}

.plot svg {
  width: 100%;
  height: 100%;
  display: block;
  cursor: crosshair;
}

.grid-line {
  stroke: color-mix(in srgb, var(--muted) 35%, transparent);
  stroke-width: 1;
  stroke-dasharray: 4 4;
}

.axis {
  stroke: var(--line);
  stroke-width: 1;
}

.tick {
  stroke: var(--line);
  stroke-width: 1;
}

.stroke {
  fill: none;
  stroke-width: 2.25;
  stroke-linejoin: round;
  stroke-linecap: round;
}

.stroke.in {
  stroke: #3b82f6;
}

.stroke.out {
  stroke: #14b8a6;
}

.bar.in {
  fill: #3b82f6;
}

.bar.out {
  fill: #14b8a6;
}

.bar.active {
  opacity: 1;
  filter: brightness(1.08);
}

.bars .bar:not(.active) {
  opacity: 0.92;
}

.crosshair {
  stroke: color-mix(in srgb, var(--muted) 55%, transparent);
  stroke-width: 1;
  stroke-dasharray: 3 3;
}

.mark.in {
  fill: #3b82f6;
  stroke: var(--panel);
  stroke-width: 1.5;
}

.mark.out {
  fill: #14b8a6;
  stroke: var(--panel);
  stroke-width: 1.5;
}

.mark.active {
  stroke-width: 2;
}

.x-label {
  fill: var(--muted);
  font-size: 10px;
  font-family: inherit;
}

.x-label.dense {
  font-size: 9px;
}

.tooltip {
  position: absolute;
  top: 0.35rem;
  transform: translateX(-50%);
  min-width: 7.5rem;
  padding: 0.45rem 0.6rem;
  border-radius: var(--radius);
  background: var(--panel);
  border: 1px solid var(--line-strong);
  box-shadow: var(--shadow);
  pointer-events: none;
  z-index: 2;
  font-size: 0.75rem;
}

.tip-time {
  color: var(--muted);
  margin-bottom: 0.3rem;
  font-variant-numeric: tabular-nums;
}

.tip-row {
  display: flex;
  align-items: center;
  gap: 0.35rem;
  margin-top: 0.15rem;
}

.tip-row strong {
  margin-left: auto;
  font-variant-numeric: tabular-nums;
}

.tip-row i {
  width: 0.45rem;
  height: 0.45rem;
  border-radius: var(--radius);
  display: inline-block;
}

.tip-row.in i {
  background: #3b82f6;
}

.tip-row.out i {
  background: #14b8a6;
}

.legend {
  display: flex;
  gap: 1rem;
  justify-content: center;
  margin-top: 0.25rem;
  font-size: 0.8rem;
  color: var(--muted);
}

.dot {
  display: inline-block;
  width: 0.55rem;
  height: 0.55rem;
  border-radius: var(--radius);
  margin-right: 0.3rem;
  vertical-align: middle;
}

.dot.in {
  background: #3b82f6;
}

.dot.out {
  background: #14b8a6;
}
</style>
