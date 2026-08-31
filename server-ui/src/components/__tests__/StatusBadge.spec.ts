import {describe, expect, it} from 'vitest'
import {mount} from '@vue/test-utils'
import StatusBadge from '../StatusBadge.vue'
import type {StatusType, StatusSize} from '../StatusBadge.vue'

function w(props: {status: StatusType; label?: string; size?: StatusSize; dot?: boolean}) {
  return mount(StatusBadge, {props})
}

describe('StatusBadge', () => {
  describe('status modifier classes', () => {
    const statuses: StatusType[] = ['running', 'stopped', 'pending', 'error', 'info', 'online', 'offline']
    for (const status of statuses) {
      it(`applies status-badge--${status} class for status="${status}"`, () => {
        const wrapper = w({status})
        expect(wrapper.find('.status-badge').classes()).toContain(`status-badge--${status}`)
      })
    }
  })

  describe('size modifier classes', () => {
    it('applies status-badge--md class when size=md (default)', () => {
      const wrapper = w({status: 'online'})
      expect(wrapper.find('.status-badge').classes()).toContain('status-badge--md')
    })

    it('applies status-badge--sm class when size=sm', () => {
      const wrapper = w({status: 'online', size: 'sm'})
      expect(wrapper.find('.status-badge').classes()).toContain('status-badge--sm')
    })
  })

  describe('label', () => {
    it('renders provided label text', () => {
      const wrapper = w({status: 'online', label: 'Online'})
      expect(wrapper.find('.status-badge__label').text()).toBe('Online')
    })

    it('renders empty label when label prop is omitted', () => {
      const wrapper = w({status: 'offline'})
      expect(wrapper.find('.status-badge__label').text()).toBe('')
    })
  })

  describe('dot', () => {
    it('renders dot span when dot=true', () => {
      const wrapper = w({status: 'online', dot: true})
      expect(wrapper.find('.status-badge__dot').exists()).toBe(true)
    })

    it('does not render dot span when dot=false (default)', () => {
      const wrapper = w({status: 'online'})
      expect(wrapper.find('.status-badge__dot').exists()).toBe(false)
    })

    it('dot span has aria-hidden="true"', () => {
      const wrapper = w({status: 'running', dot: true})
      expect(wrapper.find('.status-badge__dot').attributes('aria-hidden')).toBe('true')
    })
  })

  describe('combined props', () => {
    it('renders sm badge with dot and label together', () => {
      const wrapper = w({status: 'error', size: 'sm', dot: true, label: 'Error'})
      expect(wrapper.find('.status-badge').classes()).toContain('status-badge--sm')
      expect(wrapper.find('.status-badge__dot').exists()).toBe(true)
      expect(wrapper.find('.status-badge__label').text()).toBe('Error')
    })

    it('both status and size classes coexist on root element', () => {
      const wrapper = w({status: 'pending', size: 'sm'})
      const classes = wrapper.find('.status-badge').classes()
      expect(classes).toContain('status-badge--pending')
      expect(classes).toContain('status-badge--sm')
    })
  })
})
