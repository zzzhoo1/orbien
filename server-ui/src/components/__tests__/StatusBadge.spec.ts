import {describe, expect, it} from 'vitest'
import {mount} from '@vue/test-utils'
import StatusBadge from '../StatusBadge.vue'

describe('StatusBadge', () => {
  it('renders the provided label', () => {
    const wrapper = mount(StatusBadge, {props: {status: 'running', label: 'Running'}})
    expect(wrapper.text()).toContain('Running')
  })

  it('renders empty label when label prop is omitted', () => {
    const wrapper = mount(StatusBadge, {props: {status: 'online'}})
    expect(wrapper.text().trim()).toBe('')
  })

  it('applies correct variant class for known statuses', () => {
    const cases: Array<[string, string]> = [
      ['online', 'status-badge--online'],
      ['offline', 'status-badge--offline'],
      ['running', 'status-badge--running'],
      ['stopped', 'status-badge--stopped'],
      ['pending', 'status-badge--pending'],
      ['error', 'status-badge--error'],
      ['info', 'status-badge--info'],
    ]
    for (const [status, cls] of cases) {
      const wrapper = mount(StatusBadge, {props: {status}})
      expect(wrapper.classes(), `status=${status}`).toContain(cls)
    }
  })

  it('applies md size class by default', () => {
    const wrapper = mount(StatusBadge, {props: {status: 'online'}})
    expect(wrapper.classes()).toContain('status-badge--md')
  })

  it('applies sm size class when size=sm', () => {
    const wrapper = mount(StatusBadge, {props: {status: 'online', size: 'sm'}})
    expect(wrapper.classes()).toContain('status-badge--sm')
  })

  it('hides dot by default', () => {
    const wrapper = mount(StatusBadge, {props: {status: 'online'}})
    expect(wrapper.find('.status-badge__dot').exists()).toBe(false)
  })

  it('renders dot when dot=true', () => {
    const wrapper = mount(StatusBadge, {props: {status: 'online', dot: true}})
    expect(wrapper.find('.status-badge__dot').exists()).toBe(true)
  })

  it('renders online label correctly', () => {
    const wrapper = mount(StatusBadge, {props: {status: 'online', label: '在线'}})
    expect(wrapper.text()).toContain('在线')
    expect(wrapper.classes()).toContain('status-badge--online')
  })

  it('renders offline label correctly', () => {
    const wrapper = mount(StatusBadge, {props: {status: 'offline', label: '离线'}})
    expect(wrapper.text()).toContain('离线')
    expect(wrapper.classes()).toContain('status-badge--offline')
  })

  // ── additional coverage ─────────────────────────────────────────────────────

  it('has status-badge base class always', () => {
    const wrapper = mount(StatusBadge, {props: {status: 'running'}})
    expect(wrapper.classes()).toContain('status-badge')
  })

  it('root element is a span', () => {
    const wrapper = mount(StatusBadge, {props: {status: 'online'}})
    expect(wrapper.element.tagName.toLowerCase()).toBe('span')
  })

  it('renders status-badge__label span', () => {
    const wrapper = mount(StatusBadge, {props: {status: 'running', label: 'OK'}})
    expect(wrapper.find('.status-badge__label').exists()).toBe(true)
    expect(wrapper.find('.status-badge__label').text()).toBe('OK')
  })

  it('dot has aria-hidden=true', () => {
    const wrapper = mount(StatusBadge, {props: {status: 'online', dot: true}})
    expect(wrapper.find('.status-badge__dot').attributes('aria-hidden')).toBe('true')
  })

  it('updates label when props change', async () => {
    const wrapper = mount(StatusBadge, {props: {status: 'online', label: 'Old'}})
    await wrapper.setProps({label: 'New'})
    expect(wrapper.find('.status-badge__label').text()).toBe('New')
  })

  it('applies both status and size classes simultaneously', () => {
    const wrapper = mount(StatusBadge, {props: {status: 'error', size: 'sm'}})
    expect(wrapper.classes()).toContain('status-badge--error')
    expect(wrapper.classes()).toContain('status-badge--sm')
  })

  it('dot is not rendered when dot=false explicitly', () => {
    const wrapper = mount(StatusBadge, {props: {status: 'running', dot: false}})
    expect(wrapper.find('.status-badge__dot').exists()).toBe(false)
  })

  it('label is empty string when omitted (no visible text)', () => {
    const wrapper = mount(StatusBadge, {props: {status: 'pending'}})
    expect(wrapper.find('.status-badge__label').text()).toBe('')
  })
})
