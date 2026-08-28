import {describe, expect, it, vi} from 'vitest'
import {mount} from '@vue/test-utils'
import StatCard from '../StatCard.vue'

vi.mock('@/components/IconBadge.vue', () => ({
  default: {
    name: 'IconBadge',
    props: ['name', 'tone'],
    template: '<div class="icon-badge-stub" :data-name="name" :data-tone="tone"/>',
  },
}))

vi.mock('@/components/AppIcon.vue', () => ({default: {}}))

describe('StatCard', () => {
  it('renders the label text', () => {
    const wrapper = mount(StatCard, {props: {label: 'Total Tunnels'}})
    expect(wrapper.find('.k').text()).toBe('Total Tunnels')
  })

  it('renders slot content in the value area', () => {
    const wrapper = mount(StatCard, {
      props: {label: 'Clients'},
      slots: {default: '42'},
    })
    expect(wrapper.find('.v').text()).toBe('42')
  })

  it('does not render IconBadge when icon prop is omitted', () => {
    const wrapper = mount(StatCard, {props: {label: 'No Icon'}})
    expect(wrapper.find('.icon-badge-stub').exists()).toBe(false)
  })

  it('renders IconBadge with correct name and tone when icon is provided', () => {
    const wrapper = mount(StatCard, {
      props: {label: 'Tunnels', icon: 'tunnel', tone: 'green'},
    })
    const badge = wrapper.find('.icon-badge-stub')
    expect(badge.exists()).toBe(true)
    expect(badge.attributes('data-name')).toBe('tunnel')
    expect(badge.attributes('data-tone')).toBe('green')
  })

  it('defaults tone to blue when tone prop is omitted', () => {
    const wrapper = mount(StatCard, {
      props: {label: 'Tunnels', icon: 'tunnel'},
    })
    expect(wrapper.find('.icon-badge-stub').attributes('data-tone')).toBe('blue')
  })

  it('renders complex slot content correctly', () => {
    const wrapper = mount(StatCard, {
      props: {label: 'Status'},
      slots: {default: '<span class="custom">Online</span>'},
    })
    expect(wrapper.find('.v .custom').text()).toBe('Online')
  })
})
