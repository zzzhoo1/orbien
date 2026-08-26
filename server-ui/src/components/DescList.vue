<script setup lang="ts">
export interface DescItem {
  key: string
  label: string
  tip?: string
}

defineProps<{
  items: DescItem[]
  columns?: number
}>()
</script>

<template>
  <dl class="desc-list" :style="{ '--desc-cols': String(columns ?? 2) }">
    <div v-for="item in items" :key="item.key" class="desc-item" :title="item.tip">
      <dt class="desc-label">{{ item.label }}</dt>
      <dd class="desc-value">
        <slot :name="item.key" :item="item"/>
      </dd>
    </div>
  </dl>
</template>

<style scoped>
.desc-list {
  display: grid;
  grid-template-columns: repeat(var(--desc-cols, 2), minmax(0, 1fr));
  gap: 0.85rem 1.25rem;
  margin: 0;
}

.desc-item {
  display: flex;
  flex-direction: column;
  gap: 0.35rem;
  min-width: 0;
  padding: 0.65rem 0.75rem;
  border-radius: var(--radius);
  background: var(--panel-hover);
  border: 1px solid transparent;
}

.desc-item:hover {
  border-color: var(--line);
}

.desc-label {
  margin: 0;
  color: var(--muted);
  font-size: 0.8rem;
  font-weight: 500;
}

.desc-value {
  margin: 0;
  min-height: 1.35rem;
  display: flex;
  align-items: center;
}

@media (max-width: 900px) {
  .desc-list {
    grid-template-columns: 1fr 1fr;
  }
}

@media (max-width: 560px) {
  .desc-list {
    grid-template-columns: 1fr;
  }
}
</style>
