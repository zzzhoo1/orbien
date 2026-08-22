<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(
  defineProps<{
    variant?: 'info' | 'success' | 'warning' | 'error'
    title?: string
    closable?: boolean
  }>(),
  {
    variant: 'info',
    title: '',
    closable: false,
  },
)

const emit = defineEmits<{
  (e: 'close'): void
}>()

const icon = computed(() => {
  switch (props.variant) {
    case 'success': return '✓'
    case 'warning': return '⚠'
    case 'error':   return '✕'
    default:        return 'ℹ'
  }
})
</script>

<template>
  <div :class="['inline-alert', `inline-alert--${variant}`]" role="alert">
    <span class="inline-alert__icon" aria-hidden="true">{{ icon }}</span>

    <div class="inline-alert__body">
      <div v-if="title" class="inline-alert__title">{{ title }}</div>
      <div v-if="$slots.default" class="inline-alert__message">
        <slot />
      </div>
    </div>

    <button
      v-if="closable"
      class="inline-alert__close"
      type="button"
      aria-label="关闭"
      @click="emit('close')"
    >
      ✕
    </button>
  </div>
</template>

<style scoped>
.inline-alert {
  display: flex;
  align-items: flex-start;
  gap: var(--space-2, 8px);
  padding: var(--space-3, 12px) var(--space-4, 16px);
  border-radius: var(--radius-md, 10px);
  font-size: 13px;
  line-height: 1.5;
  transition: opacity var(--transition-base, 0.2s ease);
}

/* ── variants ── */
.inline-alert--info {
  background: var(--color-info-bg, rgba(0, 122, 255, 0.1));
  color: var(--color-info, #007aff);
  border: 1px solid var(--color-info-border, rgba(0, 122, 255, 0.25));
}

.inline-alert--success {
  background: var(--color-success-bg, rgba(52, 199, 89, 0.1));
  color: var(--color-success, #34c759);
  border: 1px solid var(--color-success-border, rgba(52, 199, 89, 0.25));
}

.inline-alert--warning {
  background: var(--color-warning-bg, rgba(255, 159, 10, 0.1));
  color: var(--color-warning, #ff9f0a);
  border: 1px solid var(--color-warning-border, rgba(255, 159, 10, 0.25));
}

.inline-alert--error {
  background: var(--color-error-bg, rgba(255, 59, 48, 0.1));
  color: var(--color-error, #ff3b30);
  border: 1px solid var(--color-error-border, rgba(255, 59, 48, 0.25));
}

/* ── icon ── */
.inline-alert__icon {
  flex-shrink: 0;
  width: 18px;
  height: 18px;
  display: grid;
  place-items: center;
  font-size: 12px;
  font-weight: 700;
  border-radius: 50%;
  background: currentColor;
  color: #fff;
  margin-top: 1px;
}

/* ── body ── */
.inline-alert__body {
  flex: 1;
  min-width: 0;
}

.inline-alert__title {
  font-weight: 600;
  margin-bottom: 2px;
}

.inline-alert__message {
  opacity: 0.85;
}

/* ── close button ── */
.inline-alert__close {
  flex-shrink: 0;
  background: none;
  border: none;
  cursor: pointer;
  padding: 0 2px;
  font-size: 12px;
  color: currentColor;
  opacity: 0.6;
  line-height: 1;
  transition: opacity var(--transition-fast, 0.12s ease);
}

.inline-alert__close:hover {
  opacity: 1;
}
</style>
