import {describe, expect, it, vi, beforeEach} from 'vitest'
import {mount, flushPromises} from '@vue/test-utils'
import TrafficChart from '../TrafficChart.vue'

// ── mocks ──────────────────────────────────────────────────────────────────

vi.mock('@/composables/useLocale', () => ({
  useLocale: () => ({
    t: (key: string) => key,
    current: {value: 'en-US'},
    options: [],
    switchLocale: vi.fn(),
  }),
}))

vi.mock('@/utils/format', () => ({
  formatFileSize: (n: number) => `${n}B`,
}))

const mockFetchTunnel = vi.fn()
const mockFetchSystem = vi.fn()

vi.mock('@/api/client', () => ({
  fetchTunnelTraffic: (...args: unknown[]) => mockFetchTunnel(...args),
  fetchSystemTraffic: (...args: unknown[]) => mockFetchSystem(...args),
}))

// ── helpers ────────────────────────────────────────────────────────────────

function makeHistory(n = 3) {
  return Array.from({length: n}, (_, i) => ({
    date: `2024-01-${String(i + 1).padStart(2, '0')}`,
    trafficIn: (i + 1) * 100,
    trafficOut: (i + 1) * 200,
  }))
}

function okResponse(history = makeHistory(), granularity = 'day') {
  return {history, granularity}
}

function w(props: object = {}) {
  return mount(TrafficChart, {props})
}

// ── tests ──────────────────────────────────────────────────────────────────

describe('TrafficChart', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  describe('loading state', () => {
    it('shows loading text while fetch is in flight', async () => {
      let resolve!: (v: unknown) => void
      mockFetchSystem.mockReturnValue(new Promise(r => { resolve = r }))
      const wrapper = w()
      await vi.dynamicImportSettled?.()
      expect(wrapper.find('.muted').text()).toBe('traffic.loading')
      resolve(okResponse())
    })
  })

  describe('error state', () => {
    it('shows error text when fetch rejects', async () => {
      mockFetchSystem.mockRejectedValue(new Error('net error'))
      const wrapper = w()
      await flushPromises()
      expect(wrapper.find('.muted').text()).toBe('traffic.failed')
    })
  })

  describe('empty state', () => {
    it('shows empty text when history is empty array', async () => {
      mockFetchSystem.mockResolvedValue(okResponse([]))
      const wrapper = w()
      await flushPromises()
      expect(wrapper.find('.muted').text()).toBe('traffic.empty')
    })
  })

  describe('bar chart (default variant)', () => {
    it('renders bar rects when data is loaded', async () => {
      mockFetchSystem.mockResolvedValue(okResponse(makeHistory(3)))
      const wrapper = w()
      await flushPromises()
      expect(wrapper.findAll('rect.bar').length).toBeGreaterThan(0)
    })

    it('renders 2 bar rects per data point (in + out)', async () => {
      const n = 4
      mockFetchSystem.mockResolvedValue(okResponse(makeHistory(n)))
      const wrapper = w()
      await flushPromises()
      expect(wrapper.findAll('rect.bar').length).toBe(n * 2)
    })

    it('does not render line paths when variant=bar', async () => {
      mockFetchSystem.mockResolvedValue(okResponse(makeHistory(3)))
      const wrapper = w({variant: 'bar'})
      await flushPromises()
      expect(wrapper.find('path.stroke').exists()).toBe(false)
    })

    it('applies dense class when range=24h', async () => {
      mockFetchSystem.mockResolvedValue(okResponse(makeHistory(3), 'hour'))
      const wrapper = w({range: '24h'})
      await flushPromises()
      expect(wrapper.find('.traffic').classes()).toContain('dense')
    })
  })

  describe('line chart (variant=line)', () => {
    it('renders stroke path elements when variant=line', async () => {
      mockFetchSystem.mockResolvedValue(okResponse(makeHistory(3)))
      const wrapper = w({variant: 'line'})
      await flushPromises()
      expect(wrapper.findAll('path.stroke').length).toBe(2)
    })

    it('applies line class on root when variant=line', async () => {
      mockFetchSystem.mockResolvedValue(okResponse(makeHistory(3)))
      const wrapper = w({variant: 'line'})
      await flushPromises()
      expect(wrapper.find('.traffic').classes()).toContain('line')
    })

    it('does not render bar rects when variant=line', async () => {
      mockFetchSystem.mockResolvedValue(okResponse(makeHistory(3)))
      const wrapper = w({variant: 'line'})
      await flushPromises()
      expect(wrapper.find('rect.bar').exists()).toBe(false)
    })

    it('renders marker circles for each data point (in + out)', async () => {
      const n = 3
      mockFetchSystem.mockResolvedValue(okResponse(makeHistory(n)))
      const wrapper = w({variant: 'line'})
      await flushPromises()
      expect(wrapper.findAll('circle.mark').length).toBe(n * 2)
    })
  })

  describe('legend', () => {
    it('renders legend with traffic.in and traffic.out keys', async () => {
      mockFetchSystem.mockResolvedValue(okResponse(makeHistory(2)))
      const wrapper = w()
      await flushPromises()
      const legend = wrapper.find('.legend')
      expect(legend.text()).toContain('traffic.in')
      expect(legend.text()).toContain('traffic.out')
    })
  })

  describe('y-axis labels', () => {
    it('shows formatted maxVal in y-axis', async () => {
      mockFetchSystem.mockResolvedValue(okResponse([
        {date: '2024-01-01', trafficIn: 1000, trafficOut: 500},
      ]))
      const wrapper = w()
      await flushPromises()
      expect(wrapper.find('.y').text()).toContain('1000B')
    })
  })

  describe('tunnelName prop', () => {
    it('calls fetchTunnelTraffic when tunnelName is set', async () => {
      mockFetchTunnel.mockResolvedValue(okResponse(makeHistory(2)))
      const wrapper = w({tunnelName: 'my-tunnel'})
      await flushPromises()
      expect(mockFetchTunnel).toHaveBeenCalledWith('my-tunnel', '7d')
      expect(mockFetchSystem).not.toHaveBeenCalled()
    })

    it('calls fetchSystemTraffic when tunnelName is empty', async () => {
      mockFetchSystem.mockResolvedValue(okResponse(makeHistory(2)))
      const wrapper = w({tunnelName: ''})
      await flushPromises()
      expect(mockFetchSystem).toHaveBeenCalledWith('7d')
      expect(mockFetchTunnel).not.toHaveBeenCalled()
    })
  })

  describe('range prop', () => {
    it('passes range to fetchSystemTraffic', async () => {
      mockFetchSystem.mockResolvedValue(okResponse(makeHistory(2)))
      const wrapper = w({range: '24h'})
      await flushPromises()
      expect(mockFetchSystem).toHaveBeenCalledWith('24h')
    })
  })

  describe('x-axis labels', () => {
    it('renders one x-label text per data point', async () => {
      const n = 5
      mockFetchSystem.mockResolvedValue(okResponse(makeHistory(n)))
      const wrapper = w()
      await flushPromises()
      expect(wrapper.findAll('text.x-label').length).toBe(n)
    })
  })
})
