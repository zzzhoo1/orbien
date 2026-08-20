<script setup lang="ts">
export type StatusType = 'running' | 'stopped' | 'pending' | 'error' | 'info'
export type StatusSize = 'sm' | 'md'

const props = withDefaults(
  defineProps<{
    status: StatusType
    label?: string
    size?: StatusSize
    dot?: boolean
  }>(),
  {
    size: 'md',
    dot: true,
  },
)

const defaultLabels: Record<StatusType, string> = {
  running: '运行中',
  stopped: '已停止',
  pending: '处理中',
  error: '异常',
  info: '提示',
}

const displayLabel = computed(() => props.label ?? defaultLabels[props.status])

import { computed } from 'vue'
</script>

<template>
  <span
    class="status-badge"
    :class="[`status-badge--${status}`, `status-badge--${size}`]"
    :aria-label="displayLabel"
  >
    <span v-if="dot" class="status-badge__dot" aria-hidden="true" />
    <span class="status-badge__label">{{ displayLabel }}</span>
  </span>
</template>

<style scoped>
.status-badge {
  display: inline-flex;
  align-items: center;
  gap: 0.35rem;
  border-radius: var(--radius-full);
  font-weight: 600;
  white-space: nowrap;
  line-height: 1;
  transition: background-color var(--transition-fast), color var(--transition-fast);
}

/* sizes */
.status-badge--md {
  padding: 0.2rem 0.6rem;
  font-size: 0.8rem;
}

.status-badge--sm {
  padding: 0.1rem 0.45rem;
  font-size: 0.72rem;
}

/* dot */
.status-badge__dot {
  width: 0.42rem;
  height: 0.42rem;
  border-radius: 50%;
  background: currentColor;
  flex-shrink: 0;
}

.status-badge--sm .status-badge__dot {
  width: 0.35rem;
  height: 0.35rem;
}

/* variants */
.status-badge--running {
  color: var(--status-ok);
  background: var(--status-ok-soft);
}

.status-badge--stopped {
  color: var(--status-stopped);
  background: var(--status-stopped-soft);
}

.status-badge--pending {
  color: var(--status-warning);
  background: var(--status-warning-soft);
}

.status-badge--error {
  color: var(--status-error);
  background: var(--status-error-soft);
}

.status-badge--info {
  color: var(--status-info);
  background: var(--status-info-soft);
}
</style>
