import {describe, expect, it, vi} from 'vitest'
import {mount} from '@vue/test-utils'
import TrafficIO from '../TrafficIO.vue'

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

vi.mock('@/assets/icon/arrow-up.svg?raw', () => ({default: '<svg class="arrow-up"/>'})) 
vi.mock('@/assets/icon/arrow-down.svg?raw', () => ({default: '<svg class="arrow-down"/>'})) 

function w(props: object) {
  return mount(TrafficIO, {props})
}

describe('TrafficIO', () => {
  describe('values', () => {
    it('renders formatted outbound in .row.out .val', () => {
      const wrapper = w({trafficOut: 1024, trafficIn: 512})
      expect(wrapper.find('.row.out .val').text()).toBe('1024B')
    })

    it('renders formatted inbound in .row.in .val', () => {
      const wrapper = w({trafficOut: 1024, trafficIn: 512})
      expect(wrapper.find('.row.in .val').text()).toBe('512B')
    })

    it('defaults trafficIn and trafficOut to 0 when omitted', () => {
      const wrapper = w({})
      expect(wrapper.find('.row.out .val').text()).toBe('0B')
      expect(wrapper.find('.row.in .val').text()).toBe('0B')
    })

    it('treats null trafficIn as 0', () => {
      const wrapper = w({trafficIn: null, trafficOut: 200})
      expect(wrapper.find('.row.in .val').text()).toBe('0B')
    })

    it('treats null trafficOut as 0', () => {
      const wrapper = w({trafficIn: 200, trafficOut: null})
      expect(wrapper.find('.row.out .val').text()).toBe('0B')
    })
  })

  describe('layout', () => {
    it('applies stack class by default', () => {
      const wrapper = w({trafficIn: 0, trafficOut: 0})
      expect(wrapper.find('.traffic-io').classes()).toContain('stack')
    })

    it('applies inline class when layout=inline', () => {
      const wrapper = w({trafficIn: 0, trafficOut: 0, layout: 'inline'})
      expect(wrapper.find('.traffic-io').classes()).toContain('inline')
    })

    it('renders sep element only when layout=inline', () => {
      const wInline = w({trafficIn: 0, trafficOut: 0, layout: 'inline'})
      const wStack = w({trafficIn: 0, trafficOut: 0, layout: 'stack'})
      expect(wInline.find('.sep').exists()).toBe(true)
      expect(wStack.find('.sep').exists()).toBe(false)
    })
  })

  describe('variant', () => {
    it('applies plain class by default', () => {
      const wrapper = w({trafficIn: 0, trafficOut: 0})
      expect(wrapper.find('.traffic-io').classes()).toContain('plain')
    })

    it('applies chip class when variant=chip', () => {
      const wrapper = w({trafficIn: 0, trafficOut: 0, variant: 'chip'})
      expect(wrapper.find('.traffic-io').classes()).toContain('chip')
    })
  })

  describe('title tooltip', () => {
    it('title attribute contains formatted out and in values', () => {
      const wrapper = w({trafficOut: 500, trafficIn: 300})
      const title = wrapper.find('.traffic-io').attributes('title') ?? ''
      expect(title).toContain('500B')
      expect(title).toContain('300B')
    })

    it('title attribute contains t("traffic.out") and t("traffic.in") keys', () => {
      const wrapper = w({trafficOut: 1, trafficIn: 2})
      const title = wrapper.find('.traffic-io').attributes('title') ?? ''
      expect(title).toContain('traffic.out')
      expect(title).toContain('traffic.in')
    })
  })
})
