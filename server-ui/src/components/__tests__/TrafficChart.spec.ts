import {describe, expect, it, vi, beforeEach} from 'vitest'
import {mount, flushPromises} from '@vue/test-utils'
import {nextTick} from 'vue'
import TrafficChart from '../TrafficChart.vue'

// ── mock 依赖 ────────────────────────────────────────────────────────────────
vi.mock('@/composables/useLocale', () => ({
  useLocale: () => ({
    t: (key: string) => key,
    current: {value: 'en-US'},
    options: [],
    switchLocale: vi.fn(),
  }),
}))

const mockFetchSystem = vi.fn()
const mockFetchTunnel = vi.fn()

vi.mock('@/api/client', () => ({
  fetchSystemTraffic: (...args: unknown[]) => mockFetchSystem(...args),
  fetchTunnelTraffic: (...args: unknown[]) => mockFetchTunnel(...args),
}))

vi.mock('@/utils/format', () => ({
  formatFileSize: (n: number) => `${n}B`,
}))

// ── 辅助 ─────────────────────────────────────────────────────────────────────
function emptyResponse() {
  return Promise.resolve({history: [], granularity: 'day'})
}

function historyResponse(points: {date: string; trafficIn: number | string; trafficOut: number | string}[]) {
  return Promise.resolve({history: points, granularity: 'day'})
}

function mountChart(props = {}) {
  return mount(TrafficChart, {props})
}

// ── 测试 ─────────────────────────────────────────────────────────────────────
describe('TrafficChart', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('shows loading text while fetching with no cached points', async () => {
    let resolve!: (v: unknown) => void
    mockFetchSystem.mockReturnValue(new Promise((r) => {
      resolve = r
    }))
    const wrapper = mountChart()
    // 尚未 resolve，loading=true 且 points=[]；等待 loading 状态 flush 到 DOM
    await nextTick()
    expect(wrapper.text()).toContain('traffic.loading')
    resolve({history: [], granularity: 'day'})
    await flushPromises()
  })

  it('shows empty text when API returns empty history', async () => {
    mockFetchSystem.mockReturnValue(emptyResponse())
    const wrapper = mountChart()
    await flushPromises()
    expect(wrapper.text()).toContain('traffic.empty')
  })

  it('shows error text and clears points when API throws', async () => {
    mockFetchSystem.mockRejectedValue(new Error('network error'))
    const wrapper = mountChart()
    await flushPromises()
    expect(wrapper.text()).toContain('traffic.failed')
    expect(wrapper.find('svg').exists()).toBe(false)
  })

  it('calls fetchTunnelTraffic when tunnelName is provided', async () => {
    mockFetchTunnel.mockReturnValue(emptyResponse())
    mountChart({tunnelName: 'my-tunnel'})
    await flushPromises()
    expect(mockFetchTunnel).toHaveBeenCalledWith('my-tunnel', '7d')
    expect(mockFetchSystem).not.toHaveBeenCalled()
  })

  it('calls fetchSystemTraffic when tunnelName is empty', async () => {
    mockFetchSystem.mockReturnValue(emptyResponse())
    mountChart()
    await flushPromises()
    expect(mockFetchSystem).toHaveBeenCalledWith('7d')
  })

  it('converts string trafficIn/Out values via Number()', async () => {
    mockFetchSystem.mockReturnValue(historyResponse([
      {date: '2026-08-01', trafficIn: '1024', trafficOut: '2048'},
    ]))
    const wrapper = mountChart()
    await flushPromises()
    expect(wrapper.find('svg').exists()).toBe(true)
    expect(wrapper.text()).toContain('1024B')
  })

  it('maxVal defaults to 100 when all points are zero', async () => {
    mockFetchSystem.mockReturnValue(historyResponse([
      {date: '2026-08-01', trafficIn: 0, trafficOut: 0},
      {date: '2026-08-02', trafficIn: 0, trafficOut: 0},
    ]))
    const wrapper = mountChart()
    await flushPromises()
    expect(wrapper.text()).toContain('100B')
    expect(wrapper.text()).toContain('50B')
  })

  it('renders bars in bar variant (default)', async () => {
    mockFetchSystem.mockReturnValue(historyResponse([
      {date: '2026-08-01', trafficIn: 500, trafficOut: 300},
    ]))
    const wrapper = mountChart({variant: 'bar'})
    await flushPromises()
    expect(wrapper.find('.bars').exists()).toBe(true)
    expect(wrapper.find('.markers').exists()).toBe(false)
  })

  it('renders markers in line variant', async () => {
    mockFetchSystem.mockReturnValue(historyResponse([
      {date: '2026-08-01', trafficIn: 500, trafficOut: 300},
    ]))
    const wrapper = mountChart({variant: 'line'})
    await flushPromises()
    expect(wrapper.find('.markers').exists()).toBe(true)
    expect(wrapper.find('.bars').exists()).toBe(false)
  })

  it('formatLabel: day granularity formats 2026-08-27 → 8-27', async () => {
    mockFetchSystem.mockReturnValue(Promise.resolve({
      history: [{date: '2026-08-27', trafficIn: 0, trafficOut: 0}],
      granularity: 'day',
    }))
    const wrapper = mountChart()
    await flushPromises()
    expect(wrapper.text()).toContain('8-27')
  })

  it('formatLabel: hour granularity formats T14:00:00 → 14:00', async () => {
    mockFetchSystem.mockReturnValue(Promise.resolve({
      history: [{date: '2026-08-27T14:00:00', trafficIn: 0, trafficOut: 0}],
      granularity: 'hour',
    }))
    const wrapper = mountChart({range: '24h'})
    await flushPromises()
    expect(wrapper.text()).toContain('14')
  })

  it('xAt returns center for single data point (no divide-by-zero)', async () => {
    mockFetchSystem.mockReturnValue(historyResponse([
      {date: '2026-08-01', trafficIn: 100, trafficOut: 50},
    ]))
    const wrapper = mountChart()
    await flushPromises()
    expect(wrapper.find('svg').exists()).toBe(true)
    const bars = wrapper.findAll('rect.bar')
    expect(bars.length).toBe(2)
  })

  it('reloads when range prop changes', async () => {
    mockFetchSystem.mockReturnValue(emptyResponse())
    const wrapper = mountChart({range: '7d'})
    await flushPromises()
    expect(mockFetchSystem).toHaveBeenCalledWith('7d')

    mockFetchSystem.mockReturnValue(emptyResponse())
    await wrapper.setProps({range: '24h'})
    await flushPromises()
    expect(mockFetchSystem).toHaveBeenCalledWith('24h')
  })
})
