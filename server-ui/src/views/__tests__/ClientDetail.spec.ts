import {describe, it, expect, vi, beforeEach, afterEach} from 'vitest'
import {mount, flushPromises} from '@vue/test-utils'
import {createPinia, setActivePinia} from 'pinia'
import {createRouter, createMemoryHistory} from 'vue-router'
import ClientDetail from '../ClientDetail.vue'
import {ApiError} from '@/api/errors'
import * as api from '@/api'

vi.mock('@/components/AppIcon.vue', () => ({
  default: {template: '<span class="stub-app-icon"/>', props: ['name']},
}))
vi.mock('@/components/OsBadge.vue', () => ({
  default: {template: '<span class="stub-os-badge"/>', props: ['os', 'arch', 'iconOnly', 'textOnly', 'size']},
}))
vi.mock('@/components/PaginationBar.vue', () => ({
  default: {template: '<div class="stub-pagination"/>', props: ['page', 'pageSize', 'total']},
}))
vi.mock('@/components/StatusBadge.vue', () => ({
  default: {template: '<span class="stub-status-badge"/>', props: ['status', 'label']},
}))
vi.mock('@/components/TrafficIO.vue', () => ({
  default: {template: '<div class="stub-traffic-io"/>', props: ['trafficIn', 'trafficOut']},
}))

vi.mock('@/composables/useLocale', () => ({
  useLocale: () => ({
    t: (key: string, params?: Record<string, unknown>) => {
      if (key === 'clients.tunnelsSearchEmpty' && params?.q) return `${key}:${String(params.q)}`
      return key
    },
  }),
}))

vi.mock('@/composables/usePresence', () => ({
  usePresence: () => ({
    isOnline: (status: unknown) => status === 'online',
    statusLabel: (status: unknown) => String(status ?? ''),
    formatSeen: (seconds: unknown, online: boolean) => `${online ? 'online' : 'offline'}:${seconds ?? 0}`,
  }),
}))

const mockShowToast = vi.fn()
vi.mock('@/composables/useToast', () => ({
  useToast: () => ({show: mockShowToast}),
}))

vi.mock('@/utils/format', () => ({
  formatTunnelEndpoint: (type: unknown, remoteAddr: unknown) => `${String(type)}:${String(remoteAddr)}`,
  isHttpTunnelType: (type: unknown) => type === 'http' || type === 'https',
}))

vi.mock('@/api', () => ({
  fetchClient: vi.fn(),
  fetchTunnels: vi.fn(),
  kickClient: vi.fn(),
}))

function makeClient(overrides: Record<string, unknown> = {}) {
  return {
    sessionId: 'client-1',
    status: 'online',
    os: 'linux',
    arch: 'amd64',
    version: '1.0.0',
    user: 'alice',
    clientIP: '10.0.0.2',
    activeConnections: 3,
    tunnelCount: 2,
    connectedSecs: 120,
    hostname: 'host-a',
    ...overrides,
  }
}

function makeTunnel(overrides: Record<string, unknown> = {}) {
  return {
    name: 'tunnel-a',
    sessionId: 'client-1',
    type: 'tcp',
    remoteAddr: '0.0.0.0:7000',
    localAddr: '127.0.0.1:3000',
    activeConnections: 4,
    todayTrafficIn: 100,
    todayTrafficOut: 200,
    status: 'online',
    ...overrides,
  }
}

function makeRouter() {
  return createRouter({
    history: createMemoryHistory(),
    routes: [
      {path: '/clients', name: 'clients', component: {template: '<div/>'}},
      {path: '/clients/:sessionId', name: 'client-detail', component: ClientDetail},
      {path: '/tunnels/:name', name: 'tunnel-detail', component: {template: '<div/>'}},
    ],
  })
}

async function mountDetail(sessionId = 'client-1') {
  const router = makeRouter()
  await router.push(`/clients/${sessionId}`)
  const wrapper = mount(ClientDetail, {global: {plugins: [createPinia(), router]}})
  await flushPromises()
  return {wrapper, router}
}

beforeEach(() => {
  setActivePinia(createPinia())
  vi.clearAllMocks()
  vi.useFakeTimers()
  vi.stubGlobal('confirm', vi.fn(() => true))
  vi.stubGlobal('history', {length: 2})
  vi.mocked(api.fetchClient).mockResolvedValue(makeClient())
  vi.mocked(api.fetchTunnels).mockResolvedValue({
    items: [makeTunnel()],
    total: 1,
    page: 1,
    pageSize: 10,
  })
  vi.mocked(api.kickClient).mockResolvedValue(undefined)
})

afterEach(() => {
  vi.useRealTimers()
  vi.unstubAllGlobals()
})

describe('ClientDetail – loading and not found', () => {
  it('shows loading state before client resolves', async () => {
    vi.mocked(api.fetchClient).mockImplementation(() => new Promise(() => {}))
    const router = makeRouter()
    await router.push('/clients/client-1')
    const wrapper = mount(ClientDetail, {global: {plugins: [createPinia(), router]}})
    expect(wrapper.text()).toContain('traffic.loading')
  })

  it('shows not found section on 404 response', async () => {
    vi.mocked(api.fetchClient).mockRejectedValue(new ApiError('http', 'not found', {status: 404}))
    const {wrapper} = await mountDetail()
    expect(wrapper.text()).toContain('clients.notFound')
    expect(wrapper.text()).toContain('clients.notFoundDesc')
  })

  it('shows not found when fetchClient returns null-like error for unknown id', async () => {
    vi.mocked(api.fetchClient).mockRejectedValue(new Error('unknown'))
    const {wrapper} = await mountDetail('ghost-session')
    expect(wrapper.text()).toContain('clients.notFound')
  })
})

describe('ClientDetail – breadcrumb and navigation', () => {
  it('renders breadcrumb current sessionId', async () => {
    const {wrapper} = await mountDetail()
    expect(wrapper.find('.crumb-current').text()).toContain('client-1')
  })

  it('back button calls router.back when history length > 1', async () => {
    const {wrapper, router} = await mountDetail()
    const backSpy = vi.spyOn(router, 'back')
    await wrapper.find('.crumb-back').trigger('click')
    expect(backSpy).toHaveBeenCalledOnce()
  })

  it('back button pushes clients route when history length <= 1', async () => {
    vi.stubGlobal('history', {length: 1})
    const {wrapper, router} = await mountDetail()
    await wrapper.find('.crumb-back').trigger('click')
    await flushPromises()
    expect(router.currentRoute.value.name).toBe('clients')
  })

  it('crumb link pushes clients route', async () => {
    const {wrapper, router} = await mountDetail()
    await wrapper.find('.crumb-link').trigger('click')
    await flushPromises()
    expect(router.currentRoute.value.name).toBe('clients')
  })

  it('not found back button pushes clients route', async () => {
    vi.mocked(api.fetchClient).mockRejectedValue(new ApiError('http', 'not found', {status: 404}))
    const {wrapper, router} = await mountDetail()
    await wrapper.find('.back-btn').trigger('click')
    await flushPromises()
    expect(router.currentRoute.value.name).toBe('clients')
  })
})

describe('ClientDetail – summary', () => {
  it('renders client summary fields', async () => {
    const {wrapper} = await mountDetail()
    expect(wrapper.text()).toContain('client-1')
    expect(wrapper.text()).toContain('v1.0.0')
    expect(wrapper.text()).toContain('alice')
    expect(wrapper.text()).toContain('10.0.0.2')
    expect(wrapper.text()).toContain('host-a')
  })

  it('renders zero fallbacks for connections and tunnels', async () => {
    vi.mocked(api.fetchClient).mockResolvedValue(makeClient({activeConnections: undefined, tunnelCount: undefined}))
    const {wrapper} = await mountDetail()
    expect(wrapper.text()).toContain('0')
  })

  it('shows kick button only when client is online', async () => {
    vi.mocked(api.fetchClient).mockResolvedValue(makeClient({status: 'offline'}))
    const {wrapper} = await mountDetail()
    expect(wrapper.find('.kick-btn').exists()).toBe(false)
  })

  it('shows disconnected label branch for offline client', async () => {
    vi.mocked(api.fetchClient).mockResolvedValue(makeClient({status: 'offline'}))
    const {wrapper} = await mountDetail()
    expect(wrapper.text()).toContain('clients.disconnected')
  })
})

describe('ClientDetail – tunnels panel', () => {
  it('renders tunnel cards from fetch result', async () => {
    vi.mocked(api.fetchTunnels).mockResolvedValue({
      items: [makeTunnel(), makeTunnel({name: 'tunnel-b'})],
      total: 2,
      page: 1,
      pageSize: 10,
    })
    const {wrapper} = await mountDetail()
    expect(wrapper.findAll('.tunnel-card')).toHaveLength(2)
  })

  it('shows tunnels empty text when no tunnels and no search term', async () => {
    vi.mocked(api.fetchTunnels).mockResolvedValue({items: [], total: 0, page: 1, pageSize: 10})
    const {wrapper} = await mountDetail()
    expect(wrapper.text()).toContain('clients.tunnelsEmpty')
  })

  it('shows search empty text when no tunnels and search term exists', async () => {
    vi.mocked(api.fetchTunnels)
      .mockResolvedValueOnce({items: [makeTunnel()], total: 1, page: 1, pageSize: 10})
      .mockResolvedValueOnce({items: [], total: 0, page: 1, pageSize: 10})
    const {wrapper} = await mountDetail()
    const input = wrapper.find('input[type="search"]')
    await input.setValue('abc')
    vi.advanceTimersByTime(300)
    await flushPromises()
    expect(wrapper.text()).toContain('clients.tunnelsSearchEmpty:abc')
  })

  it('uses tunnels.port label for tcp tunnel', async () => {
    vi.mocked(api.fetchTunnels).mockResolvedValue({items: [makeTunnel({type: 'tcp'})], total: 1, page: 1, pageSize: 10})
    const {wrapper} = await mountDetail()
    expect(wrapper.text()).toContain('tunnels.port')
  })

  it('uses tunnels.domain label for http tunnel', async () => {
    vi.mocked(api.fetchTunnels).mockResolvedValue({items: [makeTunnel({type: 'http'})], total: 1, page: 1, pageSize: 10})
    const {wrapper} = await mountDetail()
    expect(wrapper.text()).toContain('tunnels.domain')
  })

  it('renders local address and client session in tunnel card', async () => {
    const {wrapper} = await mountDetail()
    expect(wrapper.text()).toContain('127.0.0.1:3000')
    expect(wrapper.text()).toContain('client-1')
  })

  it('navigates to tunnel detail on card click', async () => {
    const {wrapper, router} = await mountDetail()
    await wrapper.find('.tunnel-card').trigger('click')
    await flushPromises()
    expect(router.currentRoute.value.name).toBe('tunnel-detail')
    expect(router.currentRoute.value.params.name).toBe('tunnel-a')
  })

  it('navigates to tunnel detail on Enter key', async () => {
    const {wrapper, router} = await mountDetail()
    await wrapper.find('.tunnel-card').trigger('keydown', {key: 'Enter'})
    await flushPromises()
    expect(router.currentRoute.value.name).toBe('tunnel-detail')
  })

  it('navigates to tunnel detail on Space key', async () => {
    const {wrapper, router} = await mountDetail()
    await wrapper.find('.tunnel-card').trigger('keydown', {key: ' '})
    await flushPromises()
    expect(router.currentRoute.value.name).toBe('tunnel-detail')
  })

  it('renders pagination only when total > 0', async () => {
    const {wrapper} = await mountDetail()
    expect(wrapper.find('.stub-pagination').exists()).toBe(true)
  })

  it('hides pagination when total is zero', async () => {
    vi.mocked(api.fetchTunnels).mockResolvedValue({items: [], total: 0, page: 1, pageSize: 10})
    const {wrapper} = await mountDetail()
    expect(wrapper.find('.stub-pagination').exists()).toBe(false)
  })
})

describe('ClientDetail – search, refresh, and cleanup', () => {
  it('debounces tunnel search and requests page 1 with query', async () => {
    const {wrapper} = await mountDetail()
    vi.mocked(api.fetchTunnels).mockClear()
    await wrapper.find('input[type="search"]').setValue('hello')
    vi.advanceTimersByTime(299)
    expect(api.fetchTunnels).not.toHaveBeenCalled()
    vi.advanceTimersByTime(1)
    await flushPromises()
    expect(api.fetchTunnels).toHaveBeenCalledWith(expect.objectContaining({page: 1, q: 'hello', sessionId: 'client-1'}))
  })

  it('refreshes periodically every 5 seconds', async () => {
    await mountDetail()
    vi.mocked(api.fetchClient).mockClear()
    vi.mocked(api.fetchTunnels).mockClear()
    vi.advanceTimersByTime(5000)
    await flushPromises()
    expect(api.fetchClient).toHaveBeenCalled()
    expect(api.fetchTunnels).toHaveBeenCalled()
  })

  it('clears interval and pending debounce on unmount', async () => {
    const clearIntervalSpy = vi.spyOn(window, 'clearInterval')
    const clearTimeoutSpy = vi.spyOn(window, 'clearTimeout')
    const {wrapper} = await mountDetail()
    await wrapper.find('input[type="search"]').setValue('bye')
    wrapper.unmount()
    expect(clearIntervalSpy).toHaveBeenCalled()
    expect(clearTimeoutSpy).toHaveBeenCalled()
  })
})

describe('ClientDetail – kick', () => {
  it('does nothing when confirm returns false', async () => {
    vi.stubGlobal('confirm', vi.fn(() => false))
    const {wrapper} = await mountDetail()
    await wrapper.find('.kick-btn').trigger('click')
    await flushPromises()
    expect(api.kickClient).not.toHaveBeenCalled()
  })

  it('calls kickClient after confirm', async () => {
    const {wrapper} = await mountDetail()
    await wrapper.find('.kick-btn').trigger('click')
    await flushPromises()
    expect(api.kickClient).toHaveBeenCalledWith('client-1')
  })

  it('shows success toast after successful kick', async () => {
    const {wrapper} = await mountDetail()
    await wrapper.find('.kick-btn').trigger('click')
    await flushPromises()
    expect(mockShowToast).toHaveBeenCalledWith('info', 'clients.kickSuccess')
  })

  it('shows failure toast when kick rejects', async () => {
    vi.mocked(api.kickClient).mockRejectedValue(new Error('boom'))
    const {wrapper} = await mountDetail()
    await wrapper.find('.kick-btn').trigger('click')
    await flushPromises()
    expect(mockShowToast).toHaveBeenCalledWith('error', 'clients.kickFailed')
  })
})
