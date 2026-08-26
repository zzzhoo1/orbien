<script lang="ts">
export type AppIconName = 'monitor' | 'link' | 'users' | 'user' | 'tunnels' | 'kick'
</script>

<script setup lang="ts">
import {computed} from 'vue'
import computerIcon from '@/assets/icon/computer.svg?raw'
import linkIcon from '@/assets/icon/link.svg?raw'
import userIcon from '@/assets/icon/user.svg?raw'
import usersIcon from '@/assets/icon/users.svg?raw'
import shareIcon from '@/assets/icon/share.svg?raw'
import kickIcon from '@/assets/icon/kick.svg?raw'

const props = defineProps<{
  name: AppIconName
}>()

const ASSET_ICONS: Record<AppIconName, string> = {
  monitor: computerIcon,
  link: linkIcon,
  user: userIcon,
  users: usersIcon,
  tunnels: shareIcon,
  kick: kickIcon,
}

const assetSvg = computed(() => ASSET_ICONS[props.name])
</script>

<template>
  <span class="app-icon-asset" aria-hidden="true" v-html="assetSvg"/>
</template>

<style scoped>
.app-icon-asset {
  width: 1em;
  height: 1em;
  display: block;
  line-height: 0;
  color: inherit;
}

.app-icon-asset :deep(svg) {
  width: 100%;
  height: 100%;
  display: block;
  fill: currentColor;
  stroke: none;
}
</style>
