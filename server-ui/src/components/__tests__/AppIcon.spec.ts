import {describe, it, expect} from 'vitest'
import {mount} from '@vue/test-utils'
import AppIcon from '../AppIcon.vue'
import type {AppIconName} from '../AppIcon.vue'

const ICON_NAMES: AppIconName[] = ['monitor', 'link', 'users', 'user', 'tunnels', 'kick']

describe('AppIcon', () => {
  it('renders a span with class app-icon-asset', () => {
    const w = mount(AppIcon, {props: {name: 'monitor'}})
    expect(w.find('span.app-icon-asset').exists()).toBe(true)
  })

  it('has aria-hidden=true', () => {
    const w = mount(AppIcon, {props: {name: 'monitor'}})
    expect(w.find('span').attributes('aria-hidden')).toBe('true')
  })

  it.each(ICON_NAMES)('renders non-empty inner HTML for icon "%s"', (name) => {
    const w = mount(AppIcon, {props: {name}})
    expect(w.find('span.app-icon-asset').html()).toBeTruthy()
    expect(w.find('span.app-icon-asset').element.innerHTML.length).toBeGreaterThan(0)
  })

  it('changes inner HTML when name prop changes', async () => {
    const w = mount(AppIcon, {props: {name: 'monitor'}})
    const before = w.find('span.app-icon-asset').element.innerHTML
    await w.setProps({name: 'link'})
    const after = w.find('span.app-icon-asset').element.innerHTML
    expect(before).not.toBe(after)
  })

  it('keeps the asset wrapper when the icon name changes', async () => {
    const w = mount(AppIcon, {props: {name: 'user'}})
    await w.setProps({name: 'tunnels'})
    expect(w.findAll('span.app-icon-asset')).toHaveLength(1)
    expect(w.find('span.app-icon-asset').attributes('aria-hidden')).toBe('true')
  })
})
