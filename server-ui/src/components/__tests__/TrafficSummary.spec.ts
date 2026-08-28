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
})
