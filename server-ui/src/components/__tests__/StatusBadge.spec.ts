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
})
