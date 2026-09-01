import {describe, it, expect, vi, beforeEach} from 'vitest'
import {mount, flushPromises} from '@vue/test-utils'
import {createPinia, setActivePinia} from 'pinia'
import {createRouter, createMemoryHistory} from 'vue-router'
import TunnelDetail from '../TunnelDetail.vue'
import * as api from '@/api'

// ── stub child components ──────────────────────────────────────────────────────
vi.mock('@/components/AppIcon.vue',     () => ({default: {template: '<span class="stub-icon"/>',          props: ['name']}}))
vi.mock('@/components/PaginationBar.vue', () => ({default: {template: '<div class="stub-pagination-bar"/>', props: ['page', 'pageSize', 'total'], emits: ['update:page', 'update:pageSize']}}))
vi.mock('@/components/SectionCard.vue', () => ({default: {template: '<div class="stub-section-card"><slot/><slot name="extra"/></div>', props: ['title']}}))
vi.mock('@/components/StatusBadge.vue', () => ({default: {template: '<span class="stub-status-badge"/>',  props: ['status','label','size']}}))
vi.mock('@/components/TrafficChart.vue',() => ({default: {template: '<div class="stub-traffic-chart"/>',  props: ['tunnelName','range','variant','refreshMs']}}))
vi.mock('@/components/TrafficIO.vue',   () => ({default: {template: '<div class="stub-traffic-io"/>',     props: ['trafficIn','trafficOut','layout']}}))

// ── mock useLocale ────────────────────────────────────────────────────────────
vi.mock('@/composables/useLocale', () => ({
  useLocale: () => ({t: (k: string, _p?: unknown) => k}),
}))

// ── mock usePresence ──────────────────────────────────────────────────────────
vi.mock('@/composables/usePresence', () => ({
  usePresence: () => ({
    isOnline: (s: unknown) => s === 'online',
    statusLabel: (s: unknown) => String(s ?? ''),
  }),
}))

// ── mock useToast ─────────────────────────────────────────────────────────────
const mockShowToast = vi.fn()
vi.mock('@/composables/useToast', () => ({
  useToast: () => ({show: mockShowToast}),
}))

// ── mock @/api ────────────────────────────────────────────────────────────────
vi.mock('@/api', () => ({
  kickProxy: vi.fn(),
  fetchConnections: vi.fn(),
}))

// ── mock dashboard store ─────────────────────────────────────────────────────
const mockStore = {tunnels: [] as unknown[]}
vi.mock('@/stores/dashboard', () => ({useDashboardStore: () => mockStore}))

// ── fixtures ──────────────────────────────────────────────────────────────────
function makeTunnel(overrides: Record<string, unknown> = {}) {
  return {
    name: 'my-tunnel',
    type: 'tcp',
    sessionId: 'sess-abc',
    remoteAddr: '0.0.0.0:7001',
    localAddr: '127.0.0.1:3000',
    status: 'online',
    activeConnections: 4,
    todayTrafficIn: 512,
    todayTrafficOut: 1024,
    lastStartTime: null as unknown,
    ...overrides,
  }
}

function makeConnPage(items: unknown[] = [], total = 0) {
  return {total, page: 1, pageSize: 10, items}
}

function makeConn(overrides: Record<string, unknown> = {}) {
  return {
    id: '1',
    remoteAddr: '1.2.3.4:54321',
    localAddr: '127.0.0.1:3000',
    connectedAt: '2026-09-01T00:00:00Z',
    trafficIn: 100,
    trafficOut: 200,
    ...overrides,
  }
}

// ── router / mount helpers ────────────────────────────────────────────────────
function makeRouter() {
  return createRouter({
    history: createMemoryHistory(),
    routes: [
      {path: '/tunnels', name: 'tunnels', component: {template: '<div/>'}},
      {path: '/tunnels/:name', name: 'tunnel-detail', component: TunnelDetail},
      {path: '/clients/:sessionId', name: 'client-detail', component: {template: '<div/>'}},
    ],
  })
}

async function mountDetail(tunnelName = 'my-tunnel') {
  const router = makeRouter()
  await router.push(`/tunnels/${tunnelName}`)
  const wrapper = mount(TunnelDetail, {
    global: {plugins: [createPinia(), router]},
  })
  await flushPromises()
  return {wrapper, router}
}

beforeEach(() => {
  setActivePinia(createPinia())
  vi.clearAllMocks()
  mockStore.tunnels = []
  vi.mocked(api.kickProxy).mockResolvedValue(undefined)
  vi.mocked(api.fetchConnections).mockResolvedValue(makeConnPage())
})

// ── tunnel not found ──────────────────────────────────────────────────────────
describe('TunnelDetail – tunnel not found', () => {
  it('renders without crashing when tunnel is not in store', async () => {
    const {wrapper} = await mountDetail()
    expect(wrapper.find('.detail').exists()).toBe(true)
  })

  it('still shows route name when tunnel is missing', async () => {
    const {wrapper} = await mountDetail('ghost-tunnel')
    expect(wrapper.text()).toContain('ghost-tunnel')
  })

  it('renders back button', async () => {
    const {wrapper} = await mountDetail()
    expect(wrapper.find('.back').exists()).toBe(true)
  })
})

// ── back navigation ───────────────────────────────────────────────────────────
describe('TunnelDetail – back navigation', () => {
  it('navigates to tunnels list when back button clicked', async () => {
    const {wrapper, router} = await mountDetail()
    await wrapper.find('.back').trigger('click')
    await flushPromises()
    expect(router.currentRoute.value.name).toBe('tunnels')
  })
})

// ── summary card ──────────────────────────────────────────────────────────────
describe('TunnelDetail – summary card', () => {
  it('renders tunnel name', async () => {
    mockStore.tunnels = [makeTunnel()]
    const {wrapper} = await mountDetail()
    expect(wrapper.text()).toContain('my-tunnel')
  })

  it('renders type badge in uppercase', async () => {
    mockStore.tunnels = [makeTunnel({type: 'http'})]
    const {wrapper} = await mountDetail()
    expect(wrapper.find('.type-badge').text()).toBe('HTTP')
  })

  it('renders localAddr', async () => {
    mockStore.tunnels = [makeTunnel()]
    const {wrapper} = await mountDetail()
    expect(wrapper.text()).toContain('127.0.0.1:3000')
  })

  it('renders activeConnections', async () => {
    mockStore.tunnels = [makeTunnel({activeConnections: 9})]
    const {wrapper} = await mountDetail()
    expect(wrapper.text()).toContain('9')
  })

  it('renders sessionId as clickable meta-client button', async () => {
    mockStore.tunnels = [makeTunnel()]
    const {wrapper} = await mountDetail()
    const btn = wrapper.find('button.meta-client')
    expect(btn.exists()).toBe(true)
    expect(btn.text()).toContain('sess-abc')
  })

  it('renders empty meta-client span when sessionId is absent', async () => {
    mockStore.tunnels = [makeTunnel({sessionId: ''})]
    const {wrapper} = await mountDetail()
    expect(wrapper.find('button.meta-client').exists()).toBe(false)
    expect(wrapper.find('span.meta-client.is-empty').exists()).toBe(true)
  })

  it('shows lastStartTime when present', async () => {
    mockStore.tunnels = [makeTunnel({lastStartTime: '2026-01-01T00:00:00Z'})]
    const {wrapper} = await mountDetail()
    expect(wrapper.text()).toContain('tunnels.lastStarted')
  })

  it('hides lastStartTime when null', async () => {
    mockStore.tunnels = [makeTunnel({lastStartTime: null})]
    const {wrapper} = await mountDetail()
    expect(wrapper.text()).not.toContain('tunnels.lastStarted')
  })

  it('shows tunnels.port label for tcp type', async () => {
    mockStore.tunnels = [makeTunnel({type: 'tcp'})]
    const {wrapper} = await mountDetail()
    expect(wrapper.text()).toContain('tunnels.port')
  })

  it('shows tunnels.domain label for http type', async () => {
    mockStore.tunnels = [makeTunnel({type: 'http'})]
    const {wrapper} = await mountDetail()
    expect(wrapper.text()).toContain('tunnels.domain')
  })

  it('shows tunnels.port label for socks5 type (not domain)', async () => {
    mockStore.tunnels = [makeTunnel({type: 'socks5', remoteAddr: '0.0.0.0:1080'})]
    const {wrapper} = await mountDetail()
    expect(wrapper.text()).toContain('tunnels.port')
    expect(wrapper.text()).not.toContain('tunnels.domain')
  })

  it('renders SOCKS5 type badge in uppercase for socks5 tunnel', async () => {
    mockStore.tunnels = [makeTunnel({type: 'socks5'})]
    const {wrapper} = await mountDetail()
    expect(wrapper.find('.type-badge').text()).toBe('SOCKS5')
  })

  it('formats socks5 remoteAddr port correctly (strips leading colon)', async () => {
    mockStore.tunnels = [makeTunnel({type: 'socks5', remoteAddr: ':1080'})]
    const {wrapper} = await mountDetail()
    expect(wrapper.text()).toContain('1080')
    expect(wrapper.text()).not.toContain(':1080')
  })
})

// ── openClient navigation ─────────────────────────────────────────────────────
describe('TunnelDetail – openClient navigation', () => {
  it('navigates to client-detail on sessionId click', async () => {
    mockStore.tunnels = [makeTunnel()]
    const {wrapper, router} = await mountDetail()
    await wrapper.find('button.meta-client').trigger('click')
    await flushPromises()
    expect(router.currentRoute.value.name).toBe('client-detail')
    expect(router.currentRoute.value.params.sessionId).toBe('sess-abc')
  })
})

// ── chart toolbar ─────────────────────────────────────────────────────────────
describe('TunnelDetail – chart toolbar', () => {
  it('bar variant is active by default', async () => {
    const {wrapper} = await mountDetail()
    const active = wrapper.find('.range-btn.active')
    expect(active.text()).toBe('traffic.chartBar')
  })

  it('switches to line variant when line button clicked', async () => {
    const {wrapper} = await mountDetail()
    const btns = wrapper.findAll('.range-btn')
    const lineBtn = btns.find(b => b.text() === 'traffic.chartLine')!
    await lineBtn.trigger('click')
    await flushPromises()
    expect(lineBtn.classes()).toContain('active')
  })

  it('24h range is active by default', async () => {
    const {wrapper} = await mountDetail()
    const activeBtns = wrapper.findAll('.range-btn.active')
    expect(activeBtns.find(b => b.text().includes('24h'))).toBeDefined()
  })

  it('switches to 7d when 7d button clicked', async () => {
    const {wrapper} = await mountDetail()
    const btn7d = wrapper.findAll('.range-btn').find(b => b.text() === 'traffic.range7d')!
    await btn7d.trigger('click')
    await flushPromises()
    expect(btn7d.classes()).toContain('active')
  })

  it('renders TrafficChart stub with correct tunnel from route param', async () => {
    mockStore.tunnels = [makeTunnel()]
    const {wrapper} = await mountDetail('my-tunnel')
    expect(wrapper.find('.stub-traffic-chart').exists()).toBe(true)
  })
})

// ── delete ────────────────────────────────────────────────────────────────────
describe('TunnelDetail – delete', () => {
  it('renders delete button', async () => {
    mockStore.tunnels = [makeTunnel()]
    const {wrapper} = await mountDetail()
    expect(wrapper.find('.delete-btn').exists()).toBe(true)
  })

  it('clicking delete button shows confirm bar', async () => {
    mockStore.tunnels = [makeTunnel()]
    const {wrapper} = await mountDetail()
    expect(wrapper.find('.confirm-bar').exists()).toBe(false)
    await wrapper.find('.delete-btn').trigger('click')
    await flushPromises()
    expect(wrapper.find('.confirm-bar').exists()).toBe(true)
    expect(wrapper.find('.delete-btn').exists()).toBe(false)
  })

  it('cancel button hides confirm bar and shows delete button again', async () => {
    mockStore.tunnels = [makeTunnel()]
    const {wrapper} = await mountDetail()
    await wrapper.find('.delete-btn').trigger('click')
    await flushPromises()
    await wrapper.find('.confirm-cancel').trigger('click')
    await flushPromises()
    expect(wrapper.find('.confirm-bar').exists()).toBe(false)
    expect(wrapper.find('.delete-btn').exists()).toBe(true)
  })

  it('ok button calls kickProxy with tunnel name', async () => {
    mockStore.tunnels = [makeTunnel()]
    const {wrapper} = await mountDetail()
    await wrapper.find('.delete-btn').trigger('click')
    await flushPromises()
    await wrapper.find('.confirm-ok').trigger('click')
    await flushPromises()
    expect(api.kickProxy).toHaveBeenCalledWith('my-tunnel')
  })

  it('shows success toast after deletion', async () => {
    mockStore.tunnels = [makeTunnel()]
    const {wrapper} = await mountDetail()
    await wrapper.find('.delete-btn').trigger('click')
    await flushPromises()
    await wrapper.find('.confirm-ok').trigger('click')
    await flushPromises()
    expect(mockShowToast).toHaveBeenCalledWith('info', 'tunnels.deleteSuccess')
  })

  it('navigates back to tunnels list after successful deletion', async () => {
    mockStore.tunnels = [makeTunnel()]
    const {wrapper, router} = await mountDetail()
    await wrapper.find('.delete-btn').trigger('click')
    await flushPromises()
    await wrapper.find('.confirm-ok').trigger('click')
    await flushPromises()
    expect(router.currentRoute.value.name).toBe('tunnels')
  })

  it('shows error toast when kickProxy throws', async () => {
    vi.mocked(api.kickProxy).mockRejectedValue(new Error('connection refused'))
    mockStore.tunnels = [makeTunnel()]
    const {wrapper} = await mountDetail()
    await wrapper.find('.delete-btn').trigger('click')
    await flushPromises()
    await wrapper.find('.confirm-ok').trigger('click')
    await flushPromises()
    expect(mockShowToast).toHaveBeenCalledWith('error', 'connection refused')
  })

  it('hides confirm bar and re-shows delete button after failed deletion', async () => {
    vi.mocked(api.kickProxy).mockRejectedValue(new Error('fail'))
    mockStore.tunnels = [makeTunnel()]
    const {wrapper} = await mountDetail()
    await wrapper.find('.delete-btn').trigger('click')
    await flushPromises()
    await wrapper.find('.confirm-ok').trigger('click')
    await flushPromises()
    expect(wrapper.find('.confirm-bar').exists()).toBe(false)
    expect(wrapper.find('.delete-btn').exists()).toBe(true)
  })
})

// ── connections panel ─────────────────────────────────────────────────────────
describe('TunnelDetail – connections panel', () => {
  it('renders connections panel header', async () => {
    const {wrapper} = await mountDetail()
    expect(wrapper.find('.conn-panel').exists()).toBe(true)
    expect(wrapper.text()).toContain('tunnels.connectionsTitle')
  })

  it('shows empty state when no connections', async () => {
    vi.mocked(api.fetchConnections).mockResolvedValue(makeConnPage([], 0))
    const {wrapper} = await mountDetail()
    expect(wrapper.find('.conn-empty').exists()).toBe(true)
    expect(wrapper.text()).toContain('tunnels.connectionsEmpty')
  })

  it('renders connection rows when connections are returned', async () => {
    vi.mocked(api.fetchConnections).mockResolvedValue(
      makeConnPage([makeConn(), makeConn({id: '2', remoteAddr: '5.6.7.8:11111'})], 2),
    )
    const {wrapper} = await mountDetail()
    expect(wrapper.findAll('.conn-row')).toHaveLength(2)
    expect(wrapper.text()).toContain('1.2.3.4:54321')
    expect(wrapper.text()).toContain('5.6.7.8:11111')
  })

  it('renders search input', async () => {
    const {wrapper} = await mountDetail()
    expect(wrapper.find('.conn-search-input').exists()).toBe(true)
  })

  it('shows connectionsSearchEmpty when search yields no results', async () => {
    vi.mocked(api.fetchConnections).mockResolvedValue(makeConnPage([], 0))
    const {wrapper} = await mountDetail()
    const input = wrapper.find('.conn-search-input')
    await input.setValue('xyz')
    await flushPromises()
    // fast-forward debounce
    vi.useFakeTimers()
    vi.runAllTimers()
    vi.useRealTimers()
    await flushPromises()
    expect(wrapper.text()).toContain('tunnels.connectionsSearchEmpty')
  })

  it('calls fetchConnections on mount with tunnel name from route', async () => {
    await mountDetail('my-tunnel')
    expect(api.fetchConnections).toHaveBeenCalledWith(
      'my-tunnel',
      expect.objectContaining({page: 1}),
    )
  })
})
