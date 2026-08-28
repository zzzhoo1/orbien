import {describe, it, expect, vi, beforeEach} from 'vitest'
import {mount, flushPromises} from '@vue/test-utils'
import {createPinia, setActivePinia} from 'pinia'
import {createRouter, createMemoryHistory} from 'vue-router'
import Tunnels from '../Tunnels.vue'
import * as api from '@/api'

// ── stub child components ──────────────────────────────────────────────────────
vi.mock('@/components/PaginationBar.vue', () => ({default: {template: '<div class="stub-pagination"/>', props: ['page','pageSize','total']}}))
vi.mock('@/components/TrafficIO.vue',     () => ({default: {template: '<div class="stub-traffic-io"/>',  props: ['trafficIn','trafficOut']}}))
vi.mock('@/components/AppIcon.vue',       () => ({default: {template: '<span class="stub-app-icon"/>',   props: ['name']}}))
vi.mock('@/components/StatusBadge.vue',   () => ({default: {template: '<span class="stub-status-badge"/>', props: ['status','label']}}))

// ── mock useLocale ────────────────────────────────────────────────────────────
vi.mock('@/composables/useLocale', () => ({
  useLocale: () => ({t: (k: string, _p?: unknown) => k}),
}))

// ── mock useToast ─────────────────────────────────────────────────────────────
const mockShowToast = vi.fn()
vi.mock('@/composables/useToast', () => ({
  useToast: () => ({show: mockShowToast}),
}))

// ── mock usePresence ──────────────────────────────────────────────────────────
vi.mock('@/composables/usePresence', () => ({
  usePresence: () => ({
    isOnline: (s: string) => s === 'online',
    statusLabel: (s: string) => s,
  }),
}))

// ── mock @/api – factory uses only vi.fn() so hoisting is safe ────────────────────────
vi.mock('@/api', () => ({kickProxy: vi.fn()}))

// ── mock dashboard store ─────────────────────────────────────────────────────────
const mockStore = {
  tunnels: [] as unknown[],
  refresh: vi.fn(),
}
vi.mock('@/stores/dashboard', () => ({
  useDashboardStore: () => mockStore,
}))

// ── fixture ─────────────────────────────────────────────────────────────────────
function makeTunnel(overrides: Record<string, unknown> = {}) {
  return {
    name: 'tun-alpha',
    type: 'tcp',
    sessionId: 'sess-1',
    remoteAddr: '0.0.0.0:6001',
    localAddr: '127.0.0.1:3000',
    status: 'online',
    activeConnections: 3,
    todayTrafficIn: 1024,
    todayTrafficOut: 2048,
    ...overrides,
  }
}

// ── mount helper ───────────────────────────────────────────────────────────────
function makeRouter() {
  return createRouter({
    history: createMemoryHistory(),
    routes: [
      {path: '/', component: {template: '<div/>'}},
      {path: '/tunnels', component: Tunnels},
      {path: '/tunnels/:name', name: 'tunnel-detail', component: {template: '<div/>'}},
    ],
  })
}

async function mountTunnels() {
  const router = makeRouter()
  await router.push('/tunnels')
  const wrapper = mount(Tunnels, {
    global: {plugins: [createPinia(), router]},
  })
  await flushPromises()
  return {wrapper, router}
}

beforeEach(() => {
  setActivePinia(createPinia())
  vi.clearAllMocks()
  mockStore.tunnels = []
  mockStore.refresh.mockResolvedValue(undefined)
  vi.mocked(api.kickProxy).mockResolvedValue(undefined)
})

// ── empty state ─────────────────────────────────────────────────────────────────
describe('Tunnels – empty state', () => {
  it('shows tunnels.empty when no tunnels', async () => {
    const {wrapper} = await mountTunnels()
    expect(wrapper.text()).toContain('tunnels.empty')
  })

  it('does not render any tunnel-card when list is empty', async () => {
    const {wrapper} = await mountTunnels()
    expect(wrapper.findAll('.tunnel-card')).toHaveLength(0)
  })
})

// ── filter chips ─────────────────────────────────────────────────────────────────
describe('Tunnels – filter chips', () => {
  it('renders 5 protocol filter chips (all,tcp,udp,http,https)', async () => {
    const {wrapper} = await mountTunnels()
    expect(wrapper.findAll('.filter-chip')).toHaveLength(5)
  })

  it('all chip is active by default', async () => {
    const {wrapper} = await mountTunnels()
    expect(wrapper.find('.filter-chip.active').text()).toContain('tunnels.filterAll')
  })

  it('typeCounts shows total tunnel count on all chip', async () => {
    mockStore.tunnels = [makeTunnel(), makeTunnel({name: 'tun-b', type: 'http'})]
    const {wrapper} = await mountTunnels()
    const chips = wrapper.findAll('.filter-chip')
    expect(chips[0].text()).toContain('2')
  })

  it('switches active chip and filters list when tcp chip clicked', async () => {
    mockStore.tunnels = [
      makeTunnel({name: 'tun-tcp', type: 'tcp'}),
      makeTunnel({name: 'tun-http', type: 'http'}),
    ]
    const {wrapper} = await mountTunnels()
    const chips = wrapper.findAll('.filter-chip')
    await chips[1].trigger('click') // tcp
    await flushPromises()
    expect(chips[1].classes()).toContain('active')
    expect(wrapper.findAll('.tunnel-card')).toHaveLength(1)
    expect(wrapper.text()).toContain('tun-tcp')
    expect(wrapper.text()).not.toContain('tun-http')
  })

  it('shows tunnels.filterEmpty when filter yields no results', async () => {
    mockStore.tunnels = [makeTunnel({type: 'tcp'})]
    const {wrapper} = await mountTunnels()
    await wrapper.findAll('.filter-chip')[3].trigger('click') // http
    await flushPromises()
    expect(wrapper.text()).toContain('tunnels.filterEmpty')
  })

  it('resets page to 1 when protocol changes', async () => {
    mockStore.tunnels = Array.from({length: 12}, (_, i) =>
      makeTunnel({name: `t${i}`, type: 'tcp'}),
    )
    const {wrapper} = await mountTunnels()
    const vm = wrapper.vm as unknown as {page: number}
    vm.page = 2
    await flushPromises()
    await wrapper.findAll('.filter-chip')[2].trigger('click') // udp
    await flushPromises()
    expect(vm.page).toBe(1)
  })
})

// ── tunnel cards ──────────────────────────────────────────────────────────────────
describe('Tunnels – tunnel cards', () => {
  it('renders tunnel name and type', async () => {
    mockStore.tunnels = [makeTunnel()]
    const {wrapper} = await mountTunnels()
    expect(wrapper.text()).toContain('tun-alpha')
    expect(wrapper.text()).toContain('TCP')
  })

  it('renders localAddr', async () => {
    mockStore.tunnels = [makeTunnel()]
    const {wrapper} = await mountTunnels()
    expect(wrapper.text()).toContain('127.0.0.1:3000')
  })

  it('renders sessionId', async () => {
    mockStore.tunnels = [makeTunnel()]
    const {wrapper} = await mountTunnels()
    expect(wrapper.text()).toContain('sess-1')
  })

  it('shows activeConnections count', async () => {
    mockStore.tunnels = [makeTunnel({activeConnections: 7})]
    const {wrapper} = await mountTunnels()
    expect(wrapper.text()).toContain('7')
  })

  it('navigates to tunnel-detail on card click', async () => {
    mockStore.tunnels = [makeTunnel()]
    const {wrapper, router} = await mountTunnels()
    await wrapper.find('.tunnel-card').trigger('click')
    await flushPromises()
    expect(router.currentRoute.value.name).toBe('tunnel-detail')
    expect(router.currentRoute.value.params.name).toBe('tun-alpha')
  })

  it('navigates on Enter keydown', async () => {
    mockStore.tunnels = [makeTunnel()]
    const {wrapper, router} = await mountTunnels()
    await wrapper.find('.tunnel-card').trigger('keydown', {key: 'Enter'})
    await flushPromises()
    expect(router.currentRoute.value.name).toBe('tunnel-detail')
  })

  it('navigates on Space keydown', async () => {
    mockStore.tunnels = [makeTunnel()]
    const {wrapper, router} = await mountTunnels()
    await wrapper.find('.tunnel-card').trigger('keydown', {key: ' '})
    await flushPromises()
    expect(router.currentRoute.value.name).toBe('tunnel-detail')
  })
})

// ── pagination ───────────────────────────────────────────────────────────────────
describe('Tunnels – pagination', () => {
  it('shows only first pageSize items on page 1', async () => {
    mockStore.tunnels = Array.from({length: 15}, (_, i) =>
      makeTunnel({name: `tun-${i}`, sessionId: `s${i}`}),
    )
    const {wrapper} = await mountTunnels()
    expect(wrapper.findAll('.tunnel-card')).toHaveLength(10)
  })

  it('clamps page to maxPage when filter reduces results', async () => {
    mockStore.tunnels = Array.from({length: 12}, (_, i) =>
      makeTunnel({name: `t${i}`, type: i < 2 ? 'http' : 'tcp'}),
    )
    const {wrapper} = await mountTunnels()
    const vm = wrapper.vm as unknown as {page: number}
    vm.page = 2
    await flushPromises()
    await wrapper.findAll('.filter-chip')[3].trigger('click') // http
    await flushPromises()
    expect(vm.page).toBe(1)
  })
})

// ── delete (kick) ─────────────────────────────────────────────────────────────────
describe('Tunnels – delete / kick', () => {
  it('calls kickProxy with tunnel name on delete button click', async () => {
    mockStore.tunnels = [makeTunnel()]
    const {wrapper} = await mountTunnels()
    await wrapper.find('.delete-btn').trigger('click')
    await flushPromises()
    expect(api.kickProxy).toHaveBeenCalledWith('tun-alpha')
  })

  it('calls store.refresh after successful kick', async () => {
    mockStore.tunnels = [makeTunnel()]
    const {wrapper} = await mountTunnels()
    await wrapper.find('.delete-btn').trigger('click')
    await flushPromises()
    expect(mockStore.refresh).toHaveBeenCalledOnce()
  })

  it('shows success toast after kick', async () => {
    mockStore.tunnels = [makeTunnel()]
    const {wrapper} = await mountTunnels()
    await wrapper.find('.delete-btn').trigger('click')
    await flushPromises()
    expect(mockShowToast).toHaveBeenCalledWith('info', 'tunnels.deleteSuccess')
  })

  it('shows error toast when kickProxy throws', async () => {
    vi.mocked(api.kickProxy).mockRejectedValue(new Error('network error'))
    mockStore.tunnels = [makeTunnel()]
    const {wrapper} = await mountTunnels()
    await wrapper.find('.delete-btn').trigger('click')
    await flushPromises()
    expect(mockShowToast).toHaveBeenCalledWith('error', 'network error')
  })

  it('delete button does not propagate click to card (no navigation)', async () => {
    mockStore.tunnels = [makeTunnel()]
    const {wrapper, router} = await mountTunnels()
    await wrapper.find('.delete-btn').trigger('click')
    await flushPromises()
    expect(router.currentRoute.value.path).toBe('/tunnels')
  })
})
