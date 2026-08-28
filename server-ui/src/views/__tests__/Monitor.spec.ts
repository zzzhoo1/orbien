import {describe, it, expect, vi, beforeEach} from 'vitest'
import {mount, flushPromises} from '@vue/test-utils'
import {createPinia, setActivePinia} from 'pinia'
import Monitor from '../Monitor.vue'

// ── stub heavy child components ─────────────────────────────────────────────────
vi.mock('@/components/TrafficChart.vue', () => ({default: {template: '<div class="stub-traffic-chart"/>'}}))
vi.mock('@/components/DonutChart.vue',   () => ({default: {template: '<div class="stub-donut-chart"/>', props: ['slices']}}))
vi.mock('@/components/TrafficSummary.vue',() => ({default: {template: '<div class="stub-traffic-summary"/>', props: ['trafficIn','trafficOut']}}))
vi.mock('@/components/ConfigValue.vue',  () => ({default: {template: '<span class="stub-config-value">{{ value }}</span>', props: ['type','value']}}))
vi.mock('@/components/StatCard.vue',     () => ({default: {template: '<div class="stub-stat-card"><slot/></div>', props: ['label','icon','tone']}}))
vi.mock('@/components/SectionCard.vue',  () => ({default: {template: '<div class="stub-section-card"><slot/><slot name="extra"/></div>', props: ['title']}}))
vi.mock('@/components/EmptyText.vue',    () => ({default: {template: '<div class="stub-empty-text"/>', props: ['title']}}))

// ── mock useLocale ────────────────────────────────────────────────────────────
vi.mock('@/composables/useLocale', () => ({
  useLocale: () => ({t: (k: string) => k}),
}))

// ── mock dashboard store ─────────────────────────────────────────────────────────
const mockStore: Record<string, unknown> = {
  info: null,
  tokens: [],
}

vi.mock('@/stores/dashboard', () => ({
  useDashboardStore: () => mockStore,
}))

// ── helpers ───────────────────────────────────────────────────────────────────
function makeInfo(overrides: Record<string, unknown> = {}) {
  return {
    version: '1.2.3',
    config: {
      listen: ':7000',
      kcpPort: 0,
      quicPort: 0,
      httpGwPort: 0,
      httpsGwPort: 0,
      rootDomain: '',
      tcpMux: true,
      tlsForce: false,
      maxConnPool: 10,
      heartbeatTimeout: 30,
    },
    status: {
      clientCounts: 2,
      totalClientCounts: 5,
      activeConnections: 8,
      totalTrafficIn: 1024,
      totalTrafficOut: 2048,
      tunnelTypeCount: {http: 3, tcp: 1},
    },
    ...overrides,
  }
}

async function mountMonitor() {
  const wrapper = mount(Monitor, {
    global: {plugins: [createPinia()]},
  })
  await flushPromises()
  return wrapper
}

beforeEach(() => {
  setActivePinia(createPinia())
  vi.clearAllMocks()
  mockStore.info = null
  mockStore.tokens = []
})

// ── empty / null state ─────────────────────────────────────────────────────────
describe('Monitor – empty state', () => {
  it('renders without crashing when store is empty', async () => {
    const w = await mountMonitor()
    expect(w.find('.monitor').exists()).toBe(true)
  })

  it('shows EmptyText for config when info is null', async () => {
    const w = await mountMonitor()
    expect(w.find('.stub-empty-text').exists()).toBe(true)
  })

  it('shows EmptyText for tokens when tokens is empty', async () => {
    const w = await mountMonitor()
    expect(w.findAll('.stub-empty-text').length).toBeGreaterThanOrEqual(1)
  })

  it('displays zero for all KPI stat cards when status is null', async () => {
    const w = await mountMonitor()
    const cards = w.findAll('.stub-stat-card')
    cards.forEach(card => expect(card.text()).toBe('0'))
  })
})

// ── KPI computed values ──────────────────────────────────────────────────────────
describe('Monitor – KPI cards', () => {
  it('renders totalClients from status.totalClientCounts', async () => {
    mockStore.info = makeInfo()
    const w = await mountMonitor()
    const cards = w.findAll('.stub-stat-card')
    expect(cards[0].text()).toBe('5')
  })

  it('renders onlineClients from status.clientCounts', async () => {
    mockStore.info = makeInfo()
    const w = await mountMonitor()
    const cards = w.findAll('.stub-stat-card')
    expect(cards[1].text()).toBe('2')
  })

  it('renders tunnelTotal as sum of tunnelTypeCount values', async () => {
    mockStore.info = makeInfo()
    const w = await mountMonitor()
    const cards = w.findAll('.stub-stat-card')
    expect(cards[2].text()).toBe('4') // http:3 + tcp:1
  })

  it('renders activeConnections', async () => {
    mockStore.info = makeInfo()
    const w = await mountMonitor()
    const cards = w.findAll('.stub-stat-card')
    expect(cards[3].text()).toBe('8')
  })

  it('totalClients falls back to onlineClients when totalClientCounts < clientCounts', async () => {
    mockStore.info = makeInfo({
      status: {clientCounts: 7, totalClientCounts: 3, activeConnections: 0, totalTrafficIn: 0, totalTrafficOut: 0, tunnelTypeCount: {}},
    })
    const w = await mountMonitor()
    expect(w.findAll('.stub-stat-card')[0].text()).toBe('7')
  })
})

// ── configFields computed ─────────────────────────────────────────────────────────
describe('Monitor – configFields', () => {
  it('renders config values when info is present', async () => {
    mockStore.info = makeInfo()
    const w = await mountMonitor()
    expect(w.findAll('.stub-config-value').length).toBeGreaterThan(0)
  })

  it('includes version when present', async () => {
    mockStore.info = makeInfo()
    const w = await mountMonitor()
    expect(w.text()).toContain('1.2.3')
  })

  it('omits version when absent', async () => {
    mockStore.info = makeInfo({version: ''})
    const w = await mountMonitor()
    expect(w.text()).not.toContain('1.2.3')
  })

  it('includes kcpPort field when port is set (non-zero)', async () => {
    mockStore.info = makeInfo({
      config: {...(makeInfo().config as object), kcpPort: 7001},
    })
    const w = await mountMonitor()
    expect(w.text()).toContain('monitor.kcpPort')
  })

  it('omits kcpPort field when port is 0', async () => {
    mockStore.info = makeInfo()
    const w = await mountMonitor()
    expect(w.text()).not.toContain('monitor.kcpPort')
  })

  it('includes rootDomain when set', async () => {
    mockStore.info = makeInfo({
      config: {...(makeInfo().config as object), rootDomain: 'example.com'},
    })
    const w = await mountMonitor()
    expect(w.text()).toContain('monitor.rootDomain')
  })
})

// ── formatHeartbeat ─────────────────────────────────────────────────────────────
describe('Monitor – formatHeartbeat (via rendered config)', () => {
  it('renders “—” when heartbeatTimeout is null', async () => {
    mockStore.info = makeInfo({
      config: {...(makeInfo().config as object), heartbeatTimeout: null},
    })
    const w = await mountMonitor()
    expect(w.text()).toContain('—')
  })

  it('renders “common.disabled” when heartbeatTimeout < 0', async () => {
    mockStore.info = makeInfo({
      config: {...(makeInfo().config as object), heartbeatTimeout: -1},
    })
    const w = await mountMonitor()
    expect(w.text()).toContain('common.disabled')
  })

  it('renders “30s” when heartbeatTimeout is 30', async () => {
    mockStore.info = makeInfo()
    const w = await mountMonitor()
    expect(w.text()).toContain('30s')
  })
})

// ── token metrics table ───────────────────────────────────────────────────────────
describe('Monitor – token metrics table', () => {
  const TOKENS = [
    {token: 'tok-a', activeConns: 5, allowedTunnels: ['t1'], allowedProtocols: ['tcp'], allowedRemotePorts: [8080]},
    {token: 'tok-b', activeConns: 1, allowedTunnels: null, allowedProtocols: null, allowedRemotePorts: null},
  ]

  it('renders token rows when tokens are present', async () => {
    mockStore.tokens = TOKENS
    const w = await mountMonitor()
    expect(w.text()).toContain('tok-a')
    expect(w.text()).toContain('tok-b')
  })

  it('sorts by activeConns desc by default', async () => {
    mockStore.tokens = TOKENS
    const w = await mountMonitor()
    const rows = w.findAll('.token-row')
    expect(rows[0].text()).toContain('tok-a')
    expect(rows[1].text()).toContain('tok-b')
  })

  it('toggles sort to asc when sort button clicked', async () => {
    mockStore.tokens = TOKENS
    const w = await mountMonitor()
    await w.find('.sort-btn').trigger('click')
    await flushPromises()
    const rows = w.findAll('.token-row')
    expect(rows[0].text()).toContain('tok-b')
    expect(rows[1].text()).toContain('tok-a')
  })

  it('shows monitor.noRestriction when allowedTunnels is null', async () => {
    mockStore.tokens = TOKENS
    const w = await mountMonitor()
    expect(w.text()).toContain('monitor.noRestriction')
  })

  it('shows joined list when allowedTunnels is set', async () => {
    mockStore.tokens = TOKENS
    const w = await mountMonitor()
    expect(w.text()).toContain('t1')
  })
})

// ── chart toolbar ─────────────────────────────────────────────────────────────────
describe('Monitor – chart toolbar', () => {
  it('seg buttons render', async () => {
    const w = await mountMonitor()
    expect(w.findAll('.seg-btn').length).toBeGreaterThanOrEqual(4)
  })

  it('line chart variant is active by default', async () => {
    const w = await mountMonitor()
    const activeBtn = w.find('.seg-btn.active')
    expect(activeBtn.text()).toBe('traffic.chartLine')
  })

  it('switches to bar when bar button clicked', async () => {
    const w = await mountMonitor()
    const btns = w.findAll('.seg-btn')
    const barBtn = btns.find(b => b.text() === 'traffic.chartBar')!
    await barBtn.trigger('click')
    await flushPromises()
    expect(barBtn.classes()).toContain('active')
  })

  it('24h range is active by default', async () => {
    const w = await mountMonitor()
    const activeRangeBtn = w.findAll('.seg-btn.active').find(b => b.text().includes('24h'))
    expect(activeRangeBtn).toBeDefined()
  })

  it('switches to 7d when 7d button clicked', async () => {
    const w = await mountMonitor()
    const btn7d = w.findAll('.seg-btn').find(b => b.text() === 'traffic.range7d')!
    await btn7d.trigger('click')
    await flushPromises()
    expect(btn7d.classes()).toContain('active')
  })
})
