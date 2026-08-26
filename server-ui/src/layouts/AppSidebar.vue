<script setup lang="ts">
import { RouterLink, useRoute } from 'vue-router'
import { NAV_ITEMS } from '@/constants/menus'
import { useLocale } from '@/composables/useLocale'
import { useSidebar } from '@/composables/useSidebar'
import computerIcon from '@/assets/icon/computer.svg?raw'
import shareIcon from '@/assets/icon/share.svg?raw'
import userIcon from '@/assets/icon/user.svg?raw'
import arrowLeftIcon from '@/assets/icon/arrow-left.svg?raw'
import arrowRightIcon from '@/assets/icon/arrow-right.svg?raw'

const { t } = useLocale()
const route = useRoute()
const { collapsed, mobileOpen, isMobile, desktopCollapsed, toggleCollapsed, closeMobile } =
  useSidebar()

const SIDEBAR_ICONS: Record<(typeof NAV_ITEMS)[number]['icon'], string> = {
  monitor: computerIcon,
  tunnels: shareIcon,
  clients: userIcon,
}

function isActive(path: string) {
  if (path === '/') return route.path === '/' || route.path === ''
  return route.path === path || route.path.startsWith(`${path}/`)
}

function onNavigate() {
  if (isMobile.value) closeMobile()
}
</script>

<template>
  <div
    v-if="isMobile && mobileOpen"
    class="sidebar-backdrop"
    aria-hidden="true"
    @click="closeMobile"
  />
  <aside
    class="sidebar"
    :class="{
      'is-collapsed': desktopCollapsed,
      'is-mobile-open': isMobile && mobileOpen,
      'is-mobile': isMobile,
    }"
    :aria-label="t('nav.menu')"
  >
    <nav class="sidebar-nav">
      <RouterLink
        v-for="item in NAV_ITEMS"
        :key="item.name"
        :to="item.path"
        class="side-link"
        :class="{ active: isActive(item.path) }"
        :title="t(`nav.${item.labelKey}`)"
        @click="onNavigate"
      >
        <span
          class="side-icon"
          aria-hidden="true"
          v-html="SIDEBAR_ICONS[item.icon]"
        />
        <span v-show="!desktopCollapsed" class="side-label">{{ t(`nav.${item.labelKey}`) }}</span>
      </RouterLink>
    </nav>

    <button
      v-if="!isMobile"
      type="button"
      class="sidebar-collapse"
      :aria-label="collapsed ? t('actions.expandSidebar') : t('actions.collapseSidebar')"
      :title="collapsed ? t('actions.expandSidebar') : t('actions.collapseSidebar')"
      @click="toggleCollapsed"
    >
      <span
        class="collapse-icon"
        aria-hidden="true"
        v-html="collapsed ? arrowRightIcon : arrowLeftIcon"
      />
    </button>
  </aside>
</template>
