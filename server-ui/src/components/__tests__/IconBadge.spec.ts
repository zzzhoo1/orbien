import {describe, expect, it, vi} from 'vitest'
import {mount} from '@vue/test-utils'
import IconBadge from '../IconBadge.vue'

vi.mock('@/components/AppIcon.vue', () => ({
  default: {
    name: 'AppIcon',
    props: ['name'],
    template: '<span class="app-icon-stub" :data-name="name"/>',
  },
}))

describe('IconBadge', () => {
  it('applies tone-blue class by default', () => {
    const w = mount(IconBadge, {props: {name: 'tunnels'}})
    expect(w.find('.icon-badge').classes()).toContain('tone-blue')
  })

  it('applies the specified tone class', () => {
    for (const tone of ['blue', 'green', 'violet', 'orange'] as const) {
      const w = mount(IconBadge, {props: {name: 'tunnels', tone}})
      expect(w.find('.icon-badge').classes()).toContain(`tone-${tone}`)
    }
  })

  it('updates the tone class when the tone prop changes', async () => {
    const w = mount(IconBadge, {props: {name: 'monitor', tone: 'green'}})
    await w.setProps({tone: 'orange'})
    expect(w.find('.icon-badge').classes()).toContain('tone-orange')
    expect(w.find('.icon-badge').classes()).not.toContain('tone-green')
  })

  it('passes the name prop through to AppIcon', () => {
    const w = mount(IconBadge, {props: {name: 'users'}})
    expect(w.find('.app-icon-stub').attributes('data-name')).toBe('users')
  })

  it('has aria-hidden on the wrapper span', () => {
    const w = mount(IconBadge, {props: {name: 'tunnels'}})
    expect(w.find('.icon-badge').attributes('aria-hidden')).toBe('true')
  })
})
