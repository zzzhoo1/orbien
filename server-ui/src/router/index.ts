import { createRouter, createWebHistory } from 'vue-router'

const router = createRouter({
    history: createWebHistory(import.meta.env.BASE_URL),
    routes: [
        {
            path: '/',
            component: () => import('@/layouts/DefaultLayout.vue'),
            children: [
                {
                    path: '',
                    name: 'dashboard',
                    component: () => import('@/views/Dashboard.vue'),
                },
                {
                    path: 'clients',
                    name: 'clients',
                    component: () => import('@/views/Clients.vue'),
                },
                {
                    path: 'clients/:sessionId',
                    name: 'client-detail',
                    component: () => import('@/views/ClientDetail.vue'),
                },
                {
                    path: 'tunnels',
                    name: 'tunnels',
                    component: () => import('@/views/Tunnels.vue'),
                },
                {
                    path: 'tokens',
                    name: 'tokens',
                    component: () => import('@/views/Tokens.vue'),
                },
                {
                    path: 'settings',
                    name: 'settings',
                    component: () => import('@/views/Settings.vue'),
                },
            ],
        },
        {
            path: '/login',
            name: 'login',
            component: () => import('@/views/Login.vue'),
        },
        {
            path: '/:pathMatch(.*)*',
            redirect: '/',
        },
    ],
})

export default router
