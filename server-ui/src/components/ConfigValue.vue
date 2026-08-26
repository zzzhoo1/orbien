<script setup lang="ts">
import {computed} from 'vue'
import EmptyText from '@/components/EmptyText.vue'
import {useLocale} from '@/composables/useLocale'
import {formatPort, formatText, isUnsetPort, isUnsetText} from '@/utils/format'

const props = withDefaults(
    defineProps<{

      type?: 'text' | 'port' | 'bool' | 'raw'
      value?: string | number | boolean | null
    }>(),
    {type: 'raw'},
)

const {t} = useLocale()

const kind = computed(() => {
  if (props.type === 'bool') {
    return {mode: 'bool' as const, on: Boolean(props.value)}
  }
  if (props.type === 'port') {
    const n = typeof props.value === 'number' ? props.value : Number(props.value)
    if (isUnsetPort(n)) return {mode: 'empty' as const}
    return {mode: 'text' as const, text: formatPort(n)!}
  }
  if (props.type === 'text') {
    const s = typeof props.value === 'string' ? props.value : String(props.value ?? '')
    if (isUnsetText(s)) return {mode: 'empty' as const}
    return {mode: 'text' as const, text: formatText(s)!}
  }

  if (props.value == null || props.value === '') return {mode: 'empty' as const}
  return {mode: 'text' as const, text: String(props.value)}
})
</script>

<template>
  <span v-if="kind.mode === 'bool'" class="bool-tag" :class="kind.on ? 'is-on' : 'is-off'">
    {{ kind.on ? t('common.enabled') : t('common.disabled') }}
  </span>
  <EmptyText v-else-if="kind.mode === 'empty'"/>
  <EmptyText v-else :empty="false">{{ kind.text }}</EmptyText>
</template>

<style scoped>
.bool-tag {
  display: inline-flex;
  align-items: center;
  gap: 0.35rem;
  padding: 0.15rem 0.55rem;
  border-radius: var(--radius-pill);
  font-size: 0.8rem;
  font-weight: 600;
}

.bool-tag::before {
  content: '';
  width: 0.45rem;
  height: 0.45rem;
  border-radius: var(--radius-circle);
  background: currentColor;
}

.bool-tag.is-on {
  color: var(--status-ok);
  background: var(--status-ok-soft);
}

.bool-tag.is-off {
  color: var(--muted);
  background: var(--panel-hover);
}
</style>
