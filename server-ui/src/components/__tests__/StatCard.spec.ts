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
})
