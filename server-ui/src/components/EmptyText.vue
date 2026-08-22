<script setup lang="ts">
import { useLocale } from '@/composables/useLocale'

withDefaults(
  defineProps<{
    empty?: boolean
    title?: string
    description?: string
    icon?: string
  }>(),
  {
    empty: true,
    title: '',
    description: '',
    icon: '',
  },
)

const { t } = useLocale()
</script>

<template>
  <!-- legacy inline usage: <EmptyText :empty="false">value</EmptyText> -->
  <span v-if="!empty" class="filled-text"><slot /></span>

  <!-- enhanced empty-state usage -->
  <div v-else class="empty-state" role="status">
    <div v-if="icon" class="empty-state__icon" aria-hidden="true">
      {{ icon }}
    </div>

    <div class="empty-state__content">
      <div class="empty-state__title">
        <slot name="title">{{ title || t('common.notConfigured') }}</slot>
      </div>

      <div v-if="description || $slots.description" class="empty-state__description">
        <slot name="description">{{ description }}</slot>
      </div>

      <div v-if="$slots.action" class="empty-state__action">
        <slot name="action" />
      </div>
    </div>
  </div>
</template>

<style scoped>
/* ── legacy inline ── */
.filled-text {
  color: var(--text);
  font-family: 'IBM Plex Mono', ui-monospace, monospace;
  font-size: 0.92rem;
  word-break: break-all;
}

/* ── enhanced empty state ── */
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: var(--space-3, 12px);
  min-height: 120px;
  padding: var(--space-6, 24px);
  color: var(--text-secondary, var(--muted));
  text-align: center;
}

.empty-state__icon {
  display: grid;
  width: 40px;
  height: 40px;
  place-items: center;
  border-radius: var(--radius-md, 12px);
  background: var(--surface-secondary, rgba(120, 120, 128, 0.12));
  font-size: 20px;
  flex-shrink: 0;
}

.empty-state__content {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--space-2, 8px);
}

.empty-state__title {
  color: var(--text, #1c1c1e);
  font-size: 14px;
  font-weight: 600;
  letter-spacing: 0.02em;
}

.empty-state__description {
  max-width: 420px;
  font-size: 13px;
  line-height: 1.5;
  color: var(--muted);
}

.empty-state__action {
  margin-top: var(--space-2, 8px);
}
</style>
