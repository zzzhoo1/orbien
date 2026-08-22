import { describe, it, expect } from 'vitest'
import { mount } from '@vue/test-utils'
import StatusBadge from '../StatusBadge.vue'

describe('StatusBadge', () => {
  it('renders the provided label', () => {
    const wrapper = mount(StatusBadge, { props: { status: 'running', label: 'Online' } })
    expect(wrapper.text()).toContain('Online')
  })

  it('falls back to default label when label prop is omitted', () => {
    const wrapper = mount(StatusBadge, { props: { status: 'stopped' } })
    expect(wrapper.text()).toContain('已停止')
  })

  it('applies correct variant class', () => {
    const wrapper = mount(StatusBadge, { props: { status: 'error' } })
    expect(wrapper.classes()).toContain('status-badge--error')
  })

  it('applies md size class by default', () => {
    const wrapper = mount(StatusBadge, { props: { status: 'running' } })
    expect(wrapper.classes()).toContain('status-badge--md')
  })

  it('applies sm size class when size=sm', () => {
    const wrapper = mount(StatusBadge, { props: { status: 'running', size: 'sm' } })
    expect(wrapper.classes()).toContain('status-badge--sm')
  })

  it('renders dot by default', () => {
    const wrapper = mount(StatusBadge, { props: { status: 'pending' } })
    expect(wrapper.find('.status-badge__dot').exists()).toBe(true)
  })

  it('hides dot when dot=false', () => {
    const wrapper = mount(StatusBadge, { props: { status: 'pending', dot: false } })
    expect(wrapper.find('.status-badge__dot').exists()).toBe(false)
  })

  it('sets aria-label to the display label', () => {
    const wrapper = mount(StatusBadge, { props: { status: 'info', label: 'Notice' } })
    expect(wrapper.attributes('aria-label')).toBe('Notice')
  })
})
