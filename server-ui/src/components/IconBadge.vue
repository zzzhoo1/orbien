<script lang="ts">
export type IconTone = 'blue' | 'green' | 'violet' | 'orange'
</script>

<script setup lang="ts">
import AppIcon, {type AppIconName} from '@/components/AppIcon.vue'

withDefaults(
    defineProps<{
      name: AppIconName
      tone?: IconTone
    }>(),
    {tone: 'blue'},
)
</script>

<template>
  <span class="icon-badge" :class="`tone-${tone}`" aria-hidden="true">
    <AppIcon :name="name"/>
  </span>
</template>

<style scoped>
.icon-badge {
  width: 2.5rem;
  height: 2.5rem;
  border-radius: var(--radius);
  display: grid;
  place-items: center;
  flex-shrink: 0;
  background: var(--icon-soft);
  color: var(--icon-fg);
  font-size: 1.25rem;
}

.tone-blue {
  --icon-fg: #3b82f6;
  --icon-soft: rgba(59, 130, 246, 0.14);
}

.tone-green {
  --icon-fg: #22c55e;
  --icon-soft: rgba(34, 197, 94, 0.14);
}

.tone-violet {
  --icon-fg: #6366f1;
  --icon-soft: rgba(99, 102, 241, 0.14);
}

.tone-orange {
  --icon-fg: #f97316;
  --icon-soft: rgba(249, 115, 22, 0.14);
}
</style>
