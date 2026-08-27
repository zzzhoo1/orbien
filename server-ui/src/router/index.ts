import { createRouter, createWebHashHistory, type RouteRecordRaw } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import Monitor from '@/views/Monitor.vue'
import Tunnels from '@/views/Tunnels.vue'
import TunnelDetail from '@/views/TunnelDetail.vue'
import Clients from '@/views/Clients.vue'
import ClientDetail from '@/views/ClientDetail.vue'
import Login from '@/views/Login.vue'

const routes: RouteRecordRaw[] = [
  { path: '/login', name: 'login', component: Login, meta: { public: true } },
  { path: '/', name: 'monitor', component: Monitor },
  { path: '/tunnels', name: 'tunnels', component: Tunnels },
  { path: '/tunnels/:name', name: 'tunnel-detail', component: TunnelDetail },
  { path: '/clients', name: 'clients', component: Clients },
  { path: '/clients/:sessionId', name: 'client-detail', component: ClientDetail },
  { path: '/overview', redirect: '/' },
]

if (import.meta.env.DEV) {
  routes.push({
    path: '/__test/status-badge',
    name: 'status-badge-test',
    component: () => import('@/views/__dev__/StatusBadgeTest.vue'),
  })
}

export const router = createRouter({
  history: createWebHashHistory(import.meta.env.BASE_URL),
  routes,
})

/**
 * Navigation guard.
 *
 * - Public routes (meta.public) always pass through.
 * - If auth store already says authenticated, allow through immediately.
 * - Otherwise call auth.fetchStatus() which hits /api/v1/auth/status
 *   (same endpoint used by the rest of the app) instead of a raw fetch
 *   against /api/v1/system/info, so the logic stays consistent.
 */
router.beforeEach(async (to) => {
  if (to.meta.public) return true
  const auth = useAuthStore()
  if (auth.authenticated) return true
  try {
    const ok = await auth.fetchStatus()
    if (ok) return true
    return { name: 'login' }
  } catch {
    return { name: 'login' }
  }
})
