import {reactive} from 'vue'
import {fetchClients, fetchTunnels, fetchSystemInfo, fetchSystemTokens, isApiError, type ApiError} from '@/api'
import {useAuthStore} from '@/stores/auth'
import {router} from '@/router'
import type {ClientInfo, TunnelInfo, SystemInfo, TokenMetricItem} from '@/types/api'

export type DashboardError =
    | { code: ApiError['code']; params?: Record<string, unknown> }
    | null

const state = reactive({
    info: null as SystemInfo | null,
    clients: [] as ClientInfo[],
    tunnels: [] as TunnelInfo[],
    tokens: [] as TokenMetricItem[],
    loading: false,
    error: null as DashboardError,
})

export function useDashboardStore() {
    async function refresh() {
        state.error = null
        try {
            const [sys, cli, tun, tokenMetrics] = await Promise.all([
                fetchSystemInfo(),
                fetchClients(),
                fetchTunnels(),
                fetchSystemTokens(),
            ])
            state.info = sys
            state.clients = cli.items ?? []
            state.tunnels = tun.items ?? []
            state.tokens = tokenMetrics.tokens ?? []
        } catch (e) {
            if (isApiError(e)) {
                // 401 — session expired; force re-login
                if (e.code === 'unauthorized') {
                    const auth = useAuthStore()
                    auth.setAuthenticated(false)
                    void router.push({ name: 'login' })
                    return
                }
                state.error = {code: e.code, params: e.params}
            } else {
                state.error = {code: 'unknown'}
            }
        }
    }

    return {
        get info() {
            return state.info
        },
        get clients() {
            return state.clients
        },
        get tunnels() {
            return state.tunnels
        },
        get proxies() {
            return state.tunnels
        },
        get tokens() {
            return state.tokens
        },
        get loading() {
            return state.loading
        },
        get error() {
            return state.error
        },
        refresh,
    }
}
