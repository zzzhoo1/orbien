import {describe, it, expect, vi, beforeEach} from 'vitest'
import {mount, flushPromises} from '@vue/test-utils'
import {createPinia, setActivePinia} from 'pinia'
import {createRouter, createMemoryHistory} from 'vue-router'
import TunnelDetail from '../TunnelDetail.vue'

// ── stub child components ──────────────────────────────────────────────────────
vi.mock('@/components/AppIcon.vue',     () => ({default: {template: '<span class="stub-icon"/>',          props: ['name']}}))
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

// ── mock dashboard store ─────────────────────────────────────────────────────────
const mockStore = {tunnels: [] as unknown[]}
vi.mock('@/stores/dashboard', () => ({useDashboardStore: () => mockStore}))

// ── fixture ─────────────────────────────────────────────────────────────────────
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

// ── router / mount helpers ─────────────────────────────────────────────────────────
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
})

// ── tunnel not found ───────────────────────────────────────────────────────────────
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

// ── back navigation ────────────────────────────────────────────────────────────────
describe('TunnelDetail – back navigation', () => {
  it('navigates to tunnels list when back button clicked', async () => {
    const {wrapper, router} = await mountDetail()
    await wrapper.find('.back').trigger('click')
    await flushPromises()
    expect(router.currentRoute.value.name).toBe('tunnels')
  })
})

// ── summary card ──────────────────────────────────────────────────────────────────
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
})

// ── openClient navigation ────────────────────────────────────────────────────────────
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

// ── chart toolbar ──────────────────────────────────────────────────────────────────
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
