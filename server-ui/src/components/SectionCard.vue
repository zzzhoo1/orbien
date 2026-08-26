<script setup lang="ts">
defineProps<{
  title: string
}>()
</script>

<template>
  <section class="section-card card">
    <header class="section-head">
      <div>
        <h2 class="section-title">{{ title }}</h2>
      </div>
      <div v-if="$slots.extra" class="section-extra">
        <slot name="extra"/>
      </div>
    </header>
    <div class="section-body">
      <slot/>
    </div>
  </section>
</template>

<style scoped>
.section-card {
  padding: 1.1rem 1.2rem 1.15rem;
}

.section-head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 1rem;
  margin-bottom: 0.95rem;
  padding-bottom: 0.75rem;
  border-bottom: 1px solid var(--line);
}

.section-title {
  margin: 0;
  font-size: 1rem;
  font-weight: 650;
  color: var(--text);
  letter-spacing: 0.01em;
}

.section-extra {
  flex-shrink: 0;
}

.section-body {
  min-width: 0;
}
</style>
