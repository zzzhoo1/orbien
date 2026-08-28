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

vi.mock('@/assets/icon/arrow-up.svg?raw', () => ({default: '<svg id="up"/>'}))  
vi.mock('@/assets/icon/arrow-down.svg?raw', () => ({default: '<svg id="down"/>'}))  

describe('TrafficIO', () => {
  it('renders formatted inbound and outbound values', () => {
    const wrapper = mount(TrafficIO, {
      props: {trafficIn: 1024, trafficOut: 2048},
    })
    expect(wrapper.find('.row.in .val').text()).toBe('1024B')
    expect(wrapper.find('.row.out .val').text()).toBe('2048B')
  })

  it('defaults both values to 0 when props are omitted', () => {
    const wrapper = mount(TrafficIO, {props: {}})
    expect(wrapper.find('.row.in .val').text()).toBe('0B')
    expect(wrapper.find('.row.out .val').text()).toBe('0B')
  })

  it('treats null props as 0', () => {
    const wrapper = mount(TrafficIO, {
      props: {trafficIn: null, trafficOut: null},
    })
    expect(wrapper.find('.row.in .val').text()).toBe('0B')
    expect(wrapper.find('.row.out .val').text()).toBe('0B')
  })

  // Vue will emit a prop-type warning for this case (Expected Number | Null, got String "abc").
  // This is intentional: the warning itself confirms the component receives illegal input and
  // the Number() || 0 guard is the defensive behaviour under test.
  it('treats non-numeric string props as 0 via Number() || 0 guard', () => {
    const wrapper = mount(TrafficIO, {
      props: {trafficIn: 'abc' as unknown as number, trafficOut: undefined},
    })
    expect(wrapper.find('.row.in .val').text()).toBe('0B')
    expect(wrapper.find('.row.out .val').text()).toBe('0B')
  })

  it('does not render separator in stack layout (default)', () => {
    const wrapper = mount(TrafficIO, {props: {}})
    expect(wrapper.find('.sep').exists()).toBe(false)
  })

  it('renders separator in inline layout', () => {
    const wrapper = mount(TrafficIO, {props: {layout: 'inline'}})
    expect(wrapper.find('.sep').exists()).toBe(true)
  })

  it('applies chip class when variant is chip', () => {
    const wrapper = mount(TrafficIO, {props: {variant: 'chip'}})
    expect(wrapper.find('.traffic-io').classes()).toContain('chip')
  })

  it('applies inline class when layout is inline', () => {
    const wrapper = mount(TrafficIO, {props: {layout: 'inline'}})
    expect(wrapper.find('.traffic-io').classes()).toContain('inline')
  })

  it('title attribute contains both in and out formatted values', () => {
    const wrapper = mount(TrafficIO, {
      props: {trafficIn: 500, trafficOut: 800},
    })
    const title = wrapper.find('.traffic-io').attributes('title') ?? ''
    expect(title).toContain('800B')
    expect(title).toContain('500B')
  })
})
