<script setup lang="ts">
import {computed} from 'vue'
import {useLocale} from '@/composables/useLocale'
import arrowLeftIcon from '@/assets/icon/arrow-left.svg?raw'
import arrowRightIcon from '@/assets/icon/arrow-right.svg?raw'

const props = withDefaults(
    defineProps<{
      total: number
      page: number
      pageSize: number
      pageSizes?: number[]
    }>(),
    {
      pageSizes: () => [10, 20, 50],
    },
)

const emit = defineEmits<{
  'update:page': [value: number]
  'update:pageSize': [value: number]
}>()

const {t} = useLocale()

const pageCount = computed(() => Math.max(1, Math.ceil(props.total / Math.max(props.pageSize, 1))))

const current = computed(() => Math.min(Math.max(1, props.page), pageCount.value))

const canPrev = computed(() => current.value > 1)
const canNext = computed(() => current.value < pageCount.value)

const pages = computed(() => {
  const total = pageCount.value
  const cur = current.value
  if (total <= 5) return Array.from({length: total}, (_, i) => i + 1)
  let start = Math.max(1, cur - 2)
  let end = start + 4
  if (end > total) {
    end = total
    start = Math.max(1, end - 4)
  }
  return Array.from({length: end - start + 1}, (_, i) => start + i)
})

function go(p: number) {
  const next = Math.min(Math.max(1, p), pageCount.value)
  if (next !== props.page) emit('update:page', next)
}

function onPageSizeChange(evt: Event) {
  const el = evt.target as HTMLSelectElement
  const size = Number(el.value)
  if (!Number.isFinite(size) || size <= 0) return
  emit('update:pageSize', size)
  emit('update:page', 1)
}
</script>

<template>
  <div class="pagination-bar" v-if="total > 0">
    <div class="pager">
      <span class="total">{{ t('common.total', {n: total}) }}</span>

      <label class="size">
        <select :value="pageSize" @change="onPageSizeChange">
          <option v-for="s in pageSizes" :key="s" :value="s">
            {{ t('common.perPage', {n: s}) }}
          </option>
        </select>
      </label>

      <div class="nav" role="navigation" :aria-label="t('common.pagination')">
        <button
            type="button"
            class="nav-btn"
            :disabled="!canPrev"
            :aria-label="t('common.prevPage')"
            @click="go(current - 1)"
        >
          <span class="nav-icon" aria-hidden="true" v-html="arrowLeftIcon"/>
        </button>

        <button
            v-for="p in pages"
            :key="p"
            type="button"
            class="page-btn"
            :class="{ active: p === current }"
            @click="go(p)"
        >
          {{ p }}
        </button>

        <button
            type="button"
            class="nav-btn"
            :disabled="!canNext"
            :aria-label="t('common.nextPage')"
            @click="go(current + 1)"
        >
          <span class="nav-icon" aria-hidden="true" v-html="arrowRightIcon"/>
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.pagination-bar {
  display: flex;
  justify-content: flex-end;
  padding: 0.25rem 0.15rem 0;
  background: transparent;
  border: 0;
  box-shadow: none;
}

.pager {
  display: inline-flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 0.85rem;
  justify-content: flex-end;
}

.total {
  color: var(--muted);
  font-size: 0.82rem;
  font-variant-numeric: tabular-nums;
}

.size select {
  appearance: none;
  font: inherit;
  font-size: 0.8rem;
  color: var(--text);
  background: var(--panel);
  border: 1px solid var(--line);
  border-radius: var(--radius);
  padding: 0.28rem 1.7rem 0.28rem 0.65rem;
  cursor: pointer;
  background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 16 16' fill='none' stroke='%2394a3b8' stroke-width='1.6' stroke-linecap='round' stroke-linejoin='round'%3E%3Cpath d='M4 6.5 8 10.5 12 6.5'/%3E%3C/svg%3E");
  background-repeat: no-repeat;
  background-position: right 0.45rem center;
  background-size: 0.85rem;
}

.size select:hover {
  border-color: var(--line-strong);
}

.size select:focus-visible {
  outline: 2px solid color-mix(in srgb, var(--accent) 45%, transparent);
  outline-offset: 1px;
}

.nav {
  display: inline-flex;
  align-items: center;
  gap: 0.15rem;
}

.nav-btn,
.page-btn {
  border: 0;
  background: transparent;
  color: var(--muted);
  font: inherit;
  font-size: 0.82rem;
  font-weight: 600;
  min-width: 1.7rem;
  height: 1.7rem;
  padding: 0 0.3rem;
  border-radius: var(--radius);
  cursor: pointer;
  display: inline-grid;
  place-items: center;
}

.nav-btn svg,
.nav-icon {
  width: 0.9rem;
  height: 0.9rem;
  display: inline-grid;
  place-items: center;
  line-height: 0;
  color: inherit;
}

.nav-icon :deep(svg) {
  width: 100%;
  height: 100%;
  display: block;
  fill: currentColor;
  stroke: none;
}

.nav-btn:hover:not(:disabled),
.page-btn:hover:not(.active) {
  color: var(--text);
  background: color-mix(in srgb, var(--muted) 10%, transparent);
}

.nav-btn:disabled {
  opacity: 0.35;
  cursor: not-allowed;
}

.page-btn.active {
  color: var(--accent-text);
}

@media (max-width: 560px) {
  .pagination-bar {
    justify-content: center;
  }

  .pager {
    justify-content: center;
  }
}
</style>
