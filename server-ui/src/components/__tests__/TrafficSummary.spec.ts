import {describe, expect, it, vi} from 'vitest'
import {mount} from '@vue/test-utils'
import TrafficSummary from '../TrafficSummary.vue'

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

vi.mock('@/assets/icon/download.svg?raw', () => ({default: '<svg id="dl"/>'}))  
vi.mock('@/assets/icon/upload.svg?raw', () => ({default: '<svg id="ul"/>'}))  

/** Parse the first integer found in an element's text, e.g. "30%" → 30 */
function pct(text: string): number {
  const m = text.match(/\d+/)
  return m ? parseInt(m[0], 10) : NaN
}

describe('TrafficSummary', () => {
  it('displays formatted total traffic', () => {
    const w = mount(TrafficSummary, {props: {trafficIn: 300, trafficOut: 700}})
    expect(w.find('.total-value').text()).toBe('1000B')
  })

  it('displays formatted inbound and outbound values', () => {
    const w = mount(TrafficSummary, {props: {trafficIn: 400, trafficOut: 600}})
    expect(w.find('.traffic-item.in .traffic-value').text()).toBe('400B')
    expect(w.find('.traffic-item.out .traffic-value').text()).toBe('600B')
  })

  it('defaults both values to 0 when props are omitted', () => {
    const w = mount(TrafficSummary, {props: {}})
    expect(w.find('.total-value').text()).toBe('0B')
  })

  it('treats null props as 0', () => {
    const w = mount(TrafficSummary, {props: {trafficIn: null, trafficOut: null}})
    expect(w.find('.total-value').text()).toBe('0B')
  })

  it('bar-in width reflects inShare percentage', () => {
    const w = mount(TrafficSummary, {props: {trafficIn: 300, trafficOut: 700}})
    expect(w.find('.bar-in').attributes('style')).toContain('width: 30%')
  })

  it('bar-out width is 100 - inShare', () => {
    const w = mount(TrafficSummary, {props: {trafficIn: 300, trafficOut: 700}})
    expect(w.find('.bar-out').attributes('style')).toContain('width: 70%')
  })

  it('defaults to 50/50 split when total is zero', () => {
    const w = mount(TrafficSummary, {props: {trafficIn: 0, trafficOut: 0}})
    expect(w.find('.bar-in').attributes('style')).toContain('width: 50%')
    expect(w.find('.bar-out').attributes('style')).toContain('width: 50%')
  })

  it('legend shows inShare and outShare percentages', () => {
    const w = mount(TrafficSummary, {props: {trafficIn: 300, trafficOut: 700}})
    expect(w.find('.leg.in').text()).toContain('30%')
    expect(w.find('.leg.out').text()).toContain('70%')
  })

  it('100% in: bar-in width=100%, bar-out width=0%', () => {
    const w = mount(TrafficSummary, {props: {trafficIn: 1000, trafficOut: 0}})
    expect(w.find('.bar-in').attributes('style')).toContain('width: 100%')
    expect(w.find('.bar-out').attributes('style')).toContain('width: 0%')
  })

  it('100% out: bar-out width=100%, bar-in width=0%', () => {
    const w = mount(TrafficSummary, {props: {trafficIn: 0, trafficOut: 1000}})
    expect(w.find('.bar-out').attributes('style')).toContain('width: 100%')
    expect(w.find('.bar-in').attributes('style')).toContain('width: 0%')
  })

  it('99/1 split: inShare=99%, outShare=1%', () => {
    const w = mount(TrafficSummary, {props: {trafficIn: 99, trafficOut: 1}})
    expect(w.find('.leg.in').text()).toContain('99%')
    expect(w.find('.leg.out').text()).toContain('1%')
  })

  it('1/99 split: inShare=1%, outShare=99%', () => {
    const w = mount(TrafficSummary, {props: {trafficIn: 1, trafficOut: 99}})
    expect(w.find('.leg.in').text()).toContain('1%')
    expect(w.find('.leg.out').text()).toContain('99%')
  })

  it('50/50 split: inShare=50%, outShare=50%', () => {
    const w = mount(TrafficSummary, {props: {trafficIn: 500, trafficOut: 500}})
    expect(w.find('.leg.in').text()).toContain('50%')
    expect(w.find('.leg.out').text()).toContain('50%')
  })

  it('total is sum of trafficIn and trafficOut', () => {
    const w = mount(TrafficSummary, {props: {trafficIn: 123, trafficOut: 456}})
    expect(w.find('.total-value').text()).toBe('579B')
  })

  it('legend in+out percentages always sum to 100', () => {
    const w = mount(TrafficSummary, {props: {trafficIn: 300, trafficOut: 700}})
    const inPct = pct(w.find('.leg.in').text())
    const outPct = pct(w.find('.leg.out').text())
    expect(inPct + outPct).toBe(100)
  })

  it('legend sum is 100 for 50/50 split', () => {
    const w = mount(TrafficSummary, {props: {trafficIn: 500, trafficOut: 500}})
    const inPct = pct(w.find('.leg.in').text())
    const outPct = pct(w.find('.leg.out').text())
    expect(inPct + outPct).toBe(100)
  })

  it('total-label uses t("traffic.total") i18n key', () => {
    const w = mount(TrafficSummary, {props: {trafficIn: 0, trafficOut: 0}})
    expect(w.find('.total-label').text()).toBe('traffic.total')
  })

  it('in traffic-label uses t("traffic.in") i18n key', () => {
    const w = mount(TrafficSummary, {props: {trafficIn: 0, trafficOut: 0}})
    expect(w.find('.traffic-item.in .traffic-label').text()).toBe('traffic.in')
  })

  it('out traffic-label uses t("traffic.out") i18n key', () => {
    const w = mount(TrafficSummary, {props: {trafficIn: 0, trafficOut: 0}})
    expect(w.find('.traffic-item.out .traffic-label').text()).toBe('traffic.out')
  })

  it('traffic-bar has role="img"', () => {
    const w = mount(TrafficSummary, {props: {trafficIn: 0, trafficOut: 0}})
    expect(w.find('.traffic-bar').attributes('role')).toBe('img')
  })

  it('traffic-bar aria-label uses t("traffic.total")', () => {
    const w = mount(TrafficSummary, {props: {trafficIn: 0, trafficOut: 0}})
    expect(w.find('.traffic-bar').attributes('aria-label')).toBe('traffic.total')
  })

  it('null trafficIn only: outbound rendered correctly', () => {
    const w = mount(TrafficSummary, {props: {trafficIn: null, trafficOut: 800}})
    expect(w.find('.traffic-item.out .traffic-value').text()).toBe('800B')
    expect(w.find('.traffic-item.in .traffic-value').text()).toBe('0B')
  })

  it('null trafficOut only: inbound rendered correctly', () => {
    const w = mount(TrafficSummary, {props: {trafficIn: 800, trafficOut: null}})
    expect(w.find('.traffic-item.in .traffic-value').text()).toBe('800B')
    expect(w.find('.traffic-item.out .traffic-value').text()).toBe('0B')
  })
})
