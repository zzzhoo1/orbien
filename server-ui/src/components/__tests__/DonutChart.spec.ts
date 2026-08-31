import {describe, expect, it, vi} from 'vitest'
import {mount} from '@vue/test-utils'
import DonutChart from '../DonutChart.vue'
import type {ChartSlice} from '../DonutChart.vue'

vi.mock('@/composables/useLocale', () => ({
  useLocale: () => ({
    t: (key: string) => key,
    current: {value: 'en-US'},
    options: [],
    switchLocale: vi.fn(),
  }),
}))

function makeSlices(overrides: Partial<ChartSlice>[] = []): ChartSlice[] {
  return overrides.map((o, i) => ({
    key: `s${i}`,
    label: `Label ${i}`,
    value: 10,
    color: '#ff0000',
    ...o,
  }))
}

describe('DonutChart', () => {
  it('renders empty ring and empty text when slices is empty', () => {
    const wrapper = mount(DonutChart, {props: {slices: []}})
    expect(wrapper.find('circle.donut-empty-ring').exists()).toBe(true)
    expect(wrapper.find('.donut-empty').exists()).toBe(true)
    expect(wrapper.find('ul.donut-legend').exists()).toBe(false)
  })

  it('shows total = 0 and empty ring when all slice values are zero', () => {
    const wrapper = mount(DonutChart, {
      props: {slices: makeSlices([{value: 0}, {value: 0}])},
    })
    expect(wrapper.find('circle.donut-empty-ring').exists()).toBe(true)
    expect(wrapper.find('.donut-total').text()).toBe('0')
    expect(wrapper.find('ul.donut-legend').exists()).toBe(true)
  })

  it('renders one arc path per positive-value slice', () => {
    const wrapper = mount(DonutChart, {
      props: {
        slices: makeSlices([
          {value: 30},
          {value: 0},
          {value: 70},
        ]),
      },
    })
    expect(wrapper.findAll('path').length).toBe(2)
  })

  it('displays correct total in center text', () => {
    const wrapper = mount(DonutChart, {
      props: {slices: makeSlices([{value: 40}, {value: 60}])},
    })
    expect(wrapper.find('.donut-total').text()).toBe('100')
  })

  it('negative values are clamped to 0 in total calculation', () => {
    const wrapper = mount(DonutChart, {
      props: {slices: makeSlices([{value: -50}, {value: 80}])},
    })
    expect(wrapper.find('.donut-total').text()).toBe('80')
    expect(wrapper.findAll('path').length).toBe(1)
  })

  it('single slice spanning 360° uses the full-circle path branch', () => {
    const wrapper = mount(DonutChart, {
      props: {slices: makeSlices([{value: 100}])},
    })
    const path = wrapper.find('path')
    expect(path.exists()).toBe(true)
    expect(path.attributes('d')).toContain('A')
  })

  it('renders legend items for every slice including zero-value ones', () => {
    const slices = makeSlices([{value: 10}, {value: 0}, {value: 30}])
    const wrapper = mount(DonutChart, {props: {slices}})
    expect(wrapper.findAll('.donut-legend li').length).toBe(3)
  })

  it('applies color from slice to arc stroke and swatch', () => {
    const wrapper = mount(DonutChart, {
      props: {
        slices: [{key: 'tcp', label: 'TCP', value: 50, color: '#3b82f6'}],
      },
    })
    expect(wrapper.find('path').attributes('stroke')).toBe('#3b82f6')
    expect(wrapper.find('.swatch').attributes('style')).toContain('rgb(59, 130, 246)')
  })

  it('size prop scales viewBox proportionally', () => {
    const wrapper = mount(DonutChart, {
      props: {
        slices: makeSlices([{value: 50}]),
        size: 100,
      },
    })
    expect(wrapper.find('svg').attributes('viewBox')).toBe('0 0 100 100')
  })

  it('uses default size 200 when size prop is omitted', () => {
    const wrapper = mount(DonutChart, {
      props: {slices: makeSlices([{value: 50}])},
    })
    expect(wrapper.find('svg').attributes('viewBox')).toBe('0 0 200 200')
  })

  it('legend renders label text for each slice', () => {
    const slices: ChartSlice[] = [
      {key: 'tcp', label: 'TCP', value: 30, color: '#f00'},
      {key: 'udp', label: 'UDP', value: 70, color: '#0f0'},
    ]
    const wrapper = mount(DonutChart, {props: {slices}})
    const names = wrapper.findAll('.name').map(n => n.text())
    expect(names).toContain('TCP')
    expect(names).toContain('UDP')
  })

  it('legend renders value count for each slice', () => {
    const slices: ChartSlice[] = [
      {key: 'tcp', label: 'TCP', value: 42, color: '#f00'},
    ]
    const wrapper = mount(DonutChart, {props: {slices}})
    expect(wrapper.find('.count').text()).toBe('42')
  })

  it('renders arc paths in slice key order (first stroke = first slice color)', () => {
    const slices: ChartSlice[] = [
      {key: 'a', label: 'A', value: 25, color: '#ff0000'},
      {key: 'b', label: 'B', value: 75, color: '#00ff00'},
    ]
    const wrapper = mount(DonutChart, {props: {slices}})
    const paths = wrapper.findAll('path')
    expect(paths[0].attributes('stroke')).toBe('#ff0000')
    expect(paths[1].attributes('stroke')).toBe('#00ff00')
  })

  it('total correctly sums three positive slices', () => {
    const wrapper = mount(DonutChart, {
      props: {slices: makeSlices([{value: 10}, {value: 20}, {value: 30}])},
    })
    expect(wrapper.find('.donut-total').text()).toBe('60')
  })

  it('no arc rendered when single slice has value = 0', () => {
    const wrapper = mount(DonutChart, {
      props: {slices: [{key: 'x', label: 'X', value: 0, color: '#red'}]},
    })
    expect(wrapper.findAll('path').length).toBe(0)
    expect(wrapper.find('circle.donut-empty-ring').exists()).toBe(true)
  })

  it('size=50 produces viewBox 0 0 50 50', () => {
    const wrapper = mount(DonutChart, {
      props: {slices: makeSlices([{value: 1}]), size: 50},
    })
    expect(wrapper.find('svg').attributes('viewBox')).toBe('0 0 50 50')
  })

  it('arc path starts with M (valid SVG path)', () => {
    const wrapper = mount(DonutChart, {
      props: {slices: makeSlices([{value: 30}, {value: 70}])},
    })
    wrapper.findAll('path').forEach(p => {
      expect(p.attributes('d')).toMatch(/^M/)
    })
  })

  it('svg has role="img" attribute', () => {
    const wrapper = mount(DonutChart, {
      props: {slices: makeSlices([{value: 10}])},
    })
    expect(wrapper.find('svg').attributes('role')).toBe('img')
  })

  it('svg aria-label is set from t("monitor.tunnelTypes")', () => {
    const wrapper = mount(DonutChart, {
      props: {slices: makeSlices([{value: 10}])},
    })
    expect(wrapper.find('svg').attributes('aria-label')).toBe('monitor.tunnelTypes')
  })

  it('donut-total-label text is set from t("monitor.chartTotal")', () => {
    const wrapper = mount(DonutChart, {
      props: {slices: makeSlices([{value: 10}])},
    })
    expect(wrapper.find('.donut-total-label').text()).toBe('monitor.chartTotal')
  })

  it('donut-empty text is set from t("common.notConfigured") when slices is empty', () => {
    const wrapper = mount(DonutChart, {props: {slices: []}})
    expect(wrapper.find('.donut-empty').text()).toBe('common.notConfigured')
  })
})
