<script setup lang="ts">
import {computed} from 'vue'

export type StatusType = 'running' | 'stopped' | 'pending' | 'error' | 'info' | 'online' | 'offline'
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
        dot: false,
    },
)

const displayLabel = computed(() => props.label ?? '')
</script>

<template>
  <span
      class="status-badge"
      :class="[`status-badge--${status}`, `status-badge--${size}`]"
  >
    <span v-if="dot" class="status-badge__dot" aria-hidden="true"/>
    <span class="status-badge__label">{{ displayLabel }}</span>
  </span>
</template>

<style scoped>
.status-badge {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 0.35rem;
  border-radius: var(--radius-pill);
  font-weight: 650;
  white-space: nowrap;
  line-height: 1.25;
}

.status-badge--md {
  min-width: 3.8rem;
  min-height: 1.85rem;
  padding: 0.28rem 0.75rem;
  font-size: 0.78rem;
}

.status-badge--sm {
  padding: 0.18rem 0.65rem;
  min-height: 1.55rem;
  font-size: 0.72rem;
}

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

/* online */
.status-badge--online {
  color: var(--status-ok);
  background: var(--status-ok-soft);
  border: 1px solid color-mix(in srgb, var(--status-ok) 22%, transparent);
}

/* offline / stopped */
.status-badge--offline,
.status-badge--stopped {
  color: var(--muted);
  background: color-mix(in srgb, var(--muted) 12%, transparent);
  border: 1px solid color-mix(in srgb, var(--muted) 18%, transparent);
}

/* running */
.status-badge--running {
  color: var(--status-ok);
  background: var(--status-ok-soft);
  border: 1px solid color-mix(in srgb, var(--status-ok) 22%, transparent);
}

/* pending */
.status-badge--pending {
  color: var(--status-warning);
  background: var(--status-warning-soft);
  border: 1px solid color-mix(in srgb, var(--status-warning) 22%, transparent);
}

/* error */
.status-badge--error {
  color: var(--status-error);
  background: var(--status-error-soft);
  border: 1px solid color-mix(in srgb, var(--status-error) 22%, transparent);
}

/* info */
.status-badge--info {
  color: var(--status-info);
  background: var(--status-info-soft);
  border: 1px solid color-mix(in srgb, var(--status-info) 22%, transparent);
}
</style>
