import {describe, it, expect, vi, beforeEach} from 'vitest'
import {createMemoryHistory, createRouter} from 'vue-router'
import type {Router} from 'vue-router'

// ── Auth store mock ───────────────────────────────────────────────────────────
const authState = {
  authenticated: false,
  fetchStatus: vi.fn(),
}
vi.mock('@/stores/auth', () => ({
  useAuthStore: () => authState,
}))

// ── View stubs ────────────────────────────────────────────────────────────────
const stub = {template: '<div/>'}
vi.mock('@/views/Monitor.vue', () => ({default: stub}))
vi.mock('@/views/Tunnels.vue', () => ({default: stub}))
vi.mock('@/views/TunnelDetail.vue', () => ({default: stub}))
vi.mock('@/views/Clients.vue', () => ({default: stub}))
vi.mock('@/views/ClientDetail.vue', () => ({default: stub}))
vi.mock('@/views/Login.vue', () => ({default: stub}))

// ── Helper: fresh router per test ──────────────────────────────────────────────
async function makeRouter(): Promise<Router> {
  vi.resetModules()
  const {router} = await import('../index')
  const mem = createMemoryHistory()
  ;(router as any).history = mem
  return router
}

beforeEach(() => {
  vi.clearAllMocks()
  authState.authenticated = false
  authState.fetchStatus.mockResolvedValue(false)
})

// ── Route definitions ────────────────────────────────────────────────────────────
describe('router – route definitions', () => {
  it('has a /login route marked as public', async () => {
    const router = await makeRouter()
    const route = router.getRoutes().find(r => r.path === '/login')
    expect(route).toBeDefined()
    expect(route?.meta.public).toBe(true)
  })

  it('has a / route named monitor', async () => {
    const router = await makeRouter()
    const route = router.getRoutes().find(r => r.path === '/')
    expect(route).toBeDefined()
    expect(route?.name).toBe('monitor')
  })

  it('has /tunnels route', async () => {
    const router = await makeRouter()
    expect(router.getRoutes().find(r => r.path === '/tunnels')).toBeDefined()
  })

  it('has /tunnels/:name route named tunnel-detail', async () => {
    const router = await makeRouter()
    const route = router.getRoutes().find(r => r.name === 'tunnel-detail')
    expect(route).toBeDefined()
    expect(route?.path).toBe('/tunnels/:name')
  })

  it('has /clients route', async () => {
    const router = await makeRouter()
    expect(router.getRoutes().find(r => r.path === '/clients')).toBeDefined()
  })

  it('has /clients/:sessionId route named client-detail', async () => {
    const router = await makeRouter()
    const route = router.getRoutes().find(r => r.name === 'client-detail')
    expect(route).toBeDefined()
    expect(route?.path).toBe('/clients/:sessionId')
  })

  it('has /overview redirect to /', async () => {
    const router = await makeRouter()
    const route = router.getRoutes().find(r => r.path === '/overview')
    expect(route).toBeDefined()
    expect(route?.redirect).toBe('/')
  })

  it('/login does NOT have a redirect', async () => {
    const router = await makeRouter()
    const route = router.getRoutes().find(r => r.path === '/login')
    expect(route?.redirect).toBeUndefined()
  })
})

// ── Navigation guard ──────────────────────────────────────────────────────────────
describe('router – navigation guard', () => {
  it('allows public route without auth', async () => {
    authState.authenticated = false
    const router = await makeRouter()
    await router.push('/login')
    expect(router.currentRoute.value.name).toBe('login')
  })

  it('allows protected route when already authenticated', async () => {
    authState.authenticated = true
    const router = await makeRouter()
    await router.push('/')
    expect(router.currentRoute.value.name).toBe('monitor')
  })

  it('allows access when fetchStatus() returns true', async () => {
    authState.authenticated = false
    authState.fetchStatus.mockResolvedValue(true)
    const router = await makeRouter()
    await router.push('/')
    expect(router.currentRoute.value.name).toBe('monitor')
  })

  it('redirects to login when fetchStatus() returns false', async () => {
    authState.authenticated = false
    authState.fetchStatus.mockResolvedValue(false)
    const router = await makeRouter()
    await router.push('/')
    expect(router.currentRoute.value.name).toBe('login')
  })

  it('redirects to login when fetchStatus() throws', async () => {
    authState.authenticated = false
    authState.fetchStatus.mockRejectedValue(new Error('network'))
    const router = await makeRouter()
    await router.push('/')
    expect(router.currentRoute.value.name).toBe('login')
  })

  it('does NOT call fetchStatus when already authenticated', async () => {
    authState.authenticated = true
    const router = await makeRouter()
    await router.push('/tunnels')
    expect(authState.fetchStatus).not.toHaveBeenCalled()
  })

  it('does NOT call fetchStatus for public routes', async () => {
    authState.authenticated = false
    const router = await makeRouter()
    await router.push('/login')
    expect(authState.fetchStatus).not.toHaveBeenCalled()
  })

  it('calls fetchStatus for protected route when not authenticated', async () => {
    authState.authenticated = false
    authState.fetchStatus.mockResolvedValue(true)
    const router = await makeRouter()
    await router.push('/clients')
    expect(authState.fetchStatus).toHaveBeenCalledOnce()
  })

  it('allows access to /tunnels/:name when authenticated', async () => {
    authState.authenticated = true
    const router = await makeRouter()
    await router.push('/tunnels/my-tunnel')
    expect(router.currentRoute.value.name).toBe('tunnel-detail')
    expect(router.currentRoute.value.params.name).toBe('my-tunnel')
  })

  it('allows access to /clients/:sessionId when authenticated', async () => {
    authState.authenticated = true
    const router = await makeRouter()
    await router.push('/clients/abc123')
    expect(router.currentRoute.value.name).toBe('client-detail')
    expect(router.currentRoute.value.params.sessionId).toBe('abc123')
  })

  it('/overview redirects to / (monitor) when authenticated', async () => {
    authState.authenticated = true
    const router = await makeRouter()
    await router.push('/overview')
    expect(router.currentRoute.value.name).toBe('monitor')
  })
})
