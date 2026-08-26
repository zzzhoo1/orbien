<script setup lang="ts">
import {useTheme} from '@/composables/useTheme'
import sunIcon from '@/assets/icon/sun.svg?raw'
import moonIcon from '@/assets/icon/moon.svg?raw'

const {isDark, label, toggle} = useTheme()
</script>

<template>
  <button
      type="button"
      class="theme-toggle"
      :aria-label="label"
      :title="label"
      @click="toggle"
  >
    <span class="theme-icon" aria-hidden="true" v-html="isDark ? sunIcon : moonIcon"/>
  </button>
</template>

<style scoped>
.theme-icon {
  display: inline-grid;
  place-items: center;
  width: 1.1rem;
  height: 1.1rem;
  line-height: 0;
}

.theme-icon :deep(svg) {
  width: 100%;
  height: 100%;
  display: block;
  fill: currentColor;
  stroke: none;
}
</style>
