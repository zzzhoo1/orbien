import {describe, it, expect} from 'vitest'
import {mount} from '@vue/test-utils'
import StatCard from '../StatCard.vue'
import IconBadge from '../IconBadge.vue'

describe('StatCard', () => {
  it('renders the label', () => {
    const w = mount(StatCard, {props: {label: 'Total clients'}})
    expect(w.find('.k').text()).toBe('Total clients')
  })

  it('renders slot content in .v', () => {
    const w = mount(StatCard, {
      props: {label: 'Count'},
      slots: {default: '42'},
    })
    expect(w.find('.v').text()).toBe('42')
  })

  it('renders IconBadge when icon prop is provided', () => {
    const w = mount(StatCard, {props: {label: 'Clients', icon: 'users'}})
    expect(w.findComponent(IconBadge).exists()).toBe(true)
  })

  it('does NOT render IconBadge when icon prop is omitted', () => {
    const w = mount(StatCard, {props: {label: 'Clients'}})
    expect(w.findComponent(IconBadge).exists()).toBe(false)
  })

  it('passes tone prop to IconBadge', () => {
    const w = mount(StatCard, {props: {label: 'x', icon: 'monitor', tone: 'green'}})
    expect(w.findComponent(IconBadge).props('tone')).toBe('green')
  })

  it('defaults tone to blue', () => {
    const w = mount(StatCard, {props: {label: 'x', icon: 'monitor'}})
    expect(w.findComponent(IconBadge).props('tone')).toBe('blue')
  })

  it('has .stat-card and .card classes', () => {
    const w = mount(StatCard, {props: {label: 'x'}})
    expect(w.find('.stat-card.card').exists()).toBe(true)
  })

  it('passes icon name to IconBadge', () => {
    const w = mount(StatCard, {props: {label: 'x', icon: 'users'}})
    expect(w.findComponent(IconBadge).props('name')).toBe('users')
  })

  it('renders .stat-top and .stat-copy containers', () => {
    const w = mount(StatCard, {props: {label: 'x'}})
    expect(w.find('.stat-top').exists()).toBe(true)
    expect(w.find('.stat-copy').exists()).toBe(true)
  })

  it('renders empty .v when default slot is omitted', () => {
    const w = mount(StatCard, {props: {label: 'x'}})
    expect(w.find('.v').text()).toBe('')
  })

  it('updates label when props change', async () => {
    const w = mount(StatCard, {props: {label: 'Old'}})
    await w.setProps({label: 'New'})
    expect(w.find('.k').text()).toBe('New')
  })

  it('updates slot content when remounted with different slot', () => {
    const w = mount(StatCard, {
      props: {label: 'Count'},
      slots: {default: '42'},
    })
    expect(w.find('.v').text()).toBe('42')
    w.unmount()
    const w2 = mount(StatCard, {
      props: {label: 'Count'},
      slots: {default: '99'},
    })
    expect(w2.find('.v').text()).toBe('99')
  })

  it('does not render stat-icon class element when icon is omitted', () => {
    const w = mount(StatCard, {props: {label: 'x'}})
    expect(w.find('.stat-icon').exists()).toBe(false)
  })

  it('renders stat-icon class element when icon is provided', () => {
    const w = mount(StatCard, {props: {label: 'x', icon: 'monitor'}})
    expect(w.find('.stat-icon').exists()).toBe(true)
  })
})
