import {createRouter, createWebHistory} from 'vue-router'
import {useAuthStore} from '@/stores/auth'

const router = createRouter({
    history: createWebHistory(import.meta.env.BASE_URL),
    routes: [
        {
            path: '/',
            name: 'monitor',
            component: () => import('@/views/Monitor.vue'),
        },
        {
            path: '/clients',
            name: 'clients',
            component: () => import('@/views/Clients.vue'),
        },
        {
            path: '/clients/:sessionId',
            name: 'client-detail',
            component: () => import('@/views/ClientDetail.vue'),
        },
        {
            path: '/tunnels',
            name: 'tunnels',
            component: () => import('@/views/Tunnels.vue'),
        },
        {
            path: '/tunnels/:name',
            name: 'tunnel-detail',
            component: () => import('@/views/TunnelDetail.vue'),
        },
        {
            path: '/login',
            name: 'login',
            meta: {public: true},
            component: () => import('@/views/Login.vue'),
        },
        {
            path: '/overview',
            redirect: '/',
        },
    ],
})

router.beforeEach(async (to) => {
    if (to.meta.public) return true
    const auth = useAuthStore()
    if (auth.authenticated) return true
    try {
        const ok = await auth.fetchStatus()
        if (ok) return true
    } catch {
        // fall through to login redirect
    }
    return {name: 'login'}
})

export default router
