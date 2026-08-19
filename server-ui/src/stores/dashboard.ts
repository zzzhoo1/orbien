import {reactive} from 'vue'
import {fetchClients, fetchProxies, fetchSystemInfo, fetchSystemTokens, isApiError, type ApiError} from '@/api'
import type {ClientInfo, ProxyInfo, SystemInfo, TokenMetricItem} from '@/types/api'

export type DashboardError =
    | { code: ApiError['code']; params?: Record<string, unknown> }
    | null

const state = reactive({
    info: null as SystemInfo | null,
    clients: [] as ClientInfo[],
    proxies: [] as ProxyInfo[],
    tokens: [] as TokenMetricItem[],
    loading: false,
    error: null as DashboardError,
})

export function useDashboardStore() {
    async function refresh() {
        state.loading = true
        state.error = null
        try {
            const [sys, cli, prox, tokenMetrics] = await Promise.all([
                fetchSystemInfo(),
                fetchClients(),
                fetchProxies(),
                fetchSystemTokens(),
            ])
            state.info = sys
            state.clients = cli.items ?? []
            state.proxies = prox.items ?? []
            state.tokens = tokenMetrics.tokens ?? []
        } catch (e) {
            if (isApiError(e)) {
                state.error = {code: e.code, params: e.params}
            } else {
                state.error = {code: 'unknown'}
            }
        } finally {
            state.loading = false
        }
    }

    return {
        get info() {
            return state.info
        },
        get clients() {
            return state.clients
        },
        get proxies() {
            return state.proxies
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
