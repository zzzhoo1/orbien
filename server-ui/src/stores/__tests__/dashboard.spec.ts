import {describe, it, expect, vi, beforeEach} from 'vitest'
import {setActivePinia, createPinia} from 'pinia'

// ── mock @/api ────────────────────────────────────────────────────────────────
const mockFetchSystemInfo = vi.fn()
const mockFetchClients = vi.fn()
const mockFetchTunnels = vi.fn()
const mockFetchSystemTokens = vi.fn()

vi.mock('@/api', () => ({
  fetchSystemInfo: (...a: unknown[]) => mockFetchSystemInfo(...a),
  fetchClients: (...a: unknown[]) => mockFetchClients(...a),
  fetchTunnels: (...a: unknown[]) => mockFetchTunnels(...a),
  fetchSystemTokens: (...a: unknown[]) => mockFetchSystemTokens(...a),
  isApiError: (e: unknown) =>
    e instanceof Error && (e as Error & {code?: string}).code !== undefined,
  ApiError: class ApiError extends Error {
    code: string
    params?: Record<string, unknown>
    constructor(code: string, params?: Record<string, unknown>) {
      super(code)
      this.code = code
      this.params = params
    }
  },
}))

// ── mock @/api/client (used by auth store) ────────────────────────────────────
vi.mock('@/api/client', () => ({
  fetchAuthStatus: vi.fn().mockResolvedValue({webauthn: false, password: true}),
}))

// ── mock @/router ─────────────────────────────────────────────────────────────
const mockRouterPush = vi.fn()
vi.mock('@/router', () => ({
  router: {push: (...a: unknown[]) => mockRouterPush(...a)},
}))

// ── fixtures ──────────────────────────────────────────────────────────────────
const SYS_INFO = {version: '1.0', os: 'linux'}
const CLIENTS = {items: [{id: 'c1'}, {id: 'c2'}]}
const TUNNELS = {items: [{name: 't1'}]}
const TOKENS = {tokens: [{token: 'tok1', activeConns: 3}]}

function successMocks() {
  mockFetchSystemInfo.mockResolvedValue(SYS_INFO)
  mockFetchClients.mockResolvedValue(CLIENTS)
  mockFetchTunnels.mockResolvedValue(TUNNELS)
  mockFetchSystemTokens.mockResolvedValue(TOKENS)
}

// Reset module registry before each test so the module-level `state` reactive
// object starts fresh. This prevents cross-test state leaks (which caused
// `loading` and `proxies` to appear as `undefined` in CI).
beforeEach(() => {
  vi.resetModules()
  setActivePinia(createPinia())
  vi.clearAllMocks()
  globalThis.fetch = vi.fn().mockResolvedValue({ok: true})
})

// ── helper ────────────────────────────────────────────────────────────────────
async function getStore() {
  const {useDashboardStore} = await import('../dashboard')
  return useDashboardStore()
}

// ── initial state ───────────────────────────────────────────────────────────
describe('useDashboardStore – initial state', () => {
  it('info is null', async () => {
    expect((await getStore()).info).toBeNull()
  })
  it('clients is empty array', async () => {
    expect((await getStore()).clients).toEqual([])
  })
  it('tunnels is empty array', async () => {
    expect((await getStore()).tunnels).toEqual([])
  })
  it('tokens is empty array', async () => {
    expect((await getStore()).tokens).toEqual([])
  })
  it('error is null', async () => {
    expect((await getStore()).error).toBeNull()
  })
  it('loading is false', async () => {
    expect((await getStore()).loading).toBe(false)
  })
})

// ── refresh success ───────────────────────────────────────────────────────────
describe('useDashboardStore – refresh success', () => {
  it('populates info after refresh', async () => {
    successMocks()
    const store = await getStore()
    await store.refresh()
    expect(store.info).toEqual(SYS_INFO)
  })

  it('populates clients after refresh', async () => {
    successMocks()
    const store = await getStore()
    await store.refresh()
    expect(store.clients).toEqual(CLIENTS.items)
  })

  it('populates tunnels after refresh', async () => {
    successMocks()
    const store = await getStore()
    await store.refresh()
    expect(store.tunnels).toEqual(TUNNELS.items)
  })

  it('proxies getter returns same value as tunnels', async () => {
    successMocks()
    const store = await getStore()
    await store.refresh()
    expect(store.proxies).toEqual(TUNNELS.items)
  })

  it('populates tokens after refresh', async () => {
    successMocks()
    const store = await getStore()
    await store.refresh()
    expect(store.tokens).toEqual(TOKENS.tokens)
  })

  it('error remains null after successful refresh', async () => {
    successMocks()
    const store = await getStore()
    await store.refresh()
    expect(store.error).toBeNull()
  })

  it('handles missing items/tokens keys gracefully (defaults to [])', async () => {
    mockFetchSystemInfo.mockResolvedValue(SYS_INFO)
    mockFetchClients.mockResolvedValue({})
    mockFetchTunnels.mockResolvedValue({})
    mockFetchSystemTokens.mockResolvedValue({})
    const store = await getStore()
    await store.refresh()
    expect(store.clients).toEqual([])
    expect(store.tunnels).toEqual([])
    expect(store.tokens).toEqual([])
  })
})

// ── error handling ────────────────────────────────────────────────────────────
describe('useDashboardStore – refresh error handling', () => {
  it('sets error.code on ApiError', async () => {
    const {ApiError} = await import('@/api')
    const err = new (ApiError as new (c: string) => Error & {code: string})('api')
    mockFetchSystemInfo.mockRejectedValue(err)
    mockFetchClients.mockResolvedValue(CLIENTS)
    mockFetchTunnels.mockResolvedValue(TUNNELS)
    mockFetchSystemTokens.mockResolvedValue(TOKENS)
    const store = await getStore()
    await store.refresh()
    expect(store.error).not.toBeNull()
    expect(store.error?.code).toBeDefined()
  })

  it('sets error.code to "unknown" for non-ApiError', async () => {
    mockFetchSystemInfo.mockRejectedValue(new Error('network failure'))
    mockFetchClients.mockResolvedValue(CLIENTS)
    mockFetchTunnels.mockResolvedValue(TUNNELS)
    mockFetchSystemTokens.mockResolvedValue(TOKENS)
    const store = await getStore()
    await store.refresh()
    expect(store.error).toEqual({code: 'unknown'})
  })

  it('clears previous error at the start of a new refresh', async () => {
    mockFetchSystemInfo.mockRejectedValue(new Error('first fail'))
    mockFetchClients.mockResolvedValue(CLIENTS)
    mockFetchTunnels.mockResolvedValue(TUNNELS)
    mockFetchSystemTokens.mockResolvedValue(TOKENS)
    const store = await getStore()
    await store.refresh()
    expect(store.error).not.toBeNull()
    successMocks()
    await store.refresh()
    expect(store.error).toBeNull()
  })

  it('redirects to login and calls setAuthenticated(false) on unauthorized ApiError', async () => {
    const {ApiError} = await import('@/api')
    const err = new (ApiError as new (c: string) => Error & {code: string})('unauthorized')
    mockFetchSystemInfo.mockRejectedValue(err)
    mockFetchClients.mockResolvedValue(CLIENTS)
    mockFetchTunnels.mockResolvedValue(TUNNELS)
    mockFetchSystemTokens.mockResolvedValue(TOKENS)
    const store = await getStore()
    const {useAuthStore} = await import('@/stores/auth')
    const auth = useAuthStore()
    auth.setAuthenticated(true, 'alice')
    await store.refresh()
    expect(auth.authenticated).toBe(false)
    expect(mockRouterPush).toHaveBeenCalledWith({name: 'login'})
  })
})
