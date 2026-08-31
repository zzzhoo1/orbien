import {describe, expect, it, vi, beforeEach} from 'vitest'
import {mount} from '@vue/test-utils'
import ThemeToggle from '../ThemeToggle.vue'

const mockToggle = vi.fn()

vi.mock('@/assets/icon/sun.svg?raw', () => ({default: '<svg id="sun"/>'}))  
vi.mock('@/assets/icon/moon.svg?raw', () => ({default: '<svg id="moon"/>'})) 

vi.mock('@/composables/useTheme')

import {useTheme} from '@/composables/useTheme'

beforeEach(() => {
  vi.clearAllMocks()
})

function setupTheme(isDark: boolean) {
  vi.mocked(useTheme).mockReturnValue({
    isDark,
    label: isDark ? 'Switch to light' : 'Switch to dark',
    toggle: mockToggle,
  })
}

describe('ThemeToggle – light mode', () => {
  it('renders moon icon when isDark is false', () => {
    setupTheme(false)
    const w = mount(ThemeToggle)
    expect(w.find('.theme-icon').html()).toContain('id="moon"')
  })

  it('aria-label and title reflect light mode label', () => {
    setupTheme(false)
    const w = mount(ThemeToggle)
    expect(w.find('button').attributes('aria-label')).toBe('Switch to dark')
    expect(w.find('button').attributes('title')).toBe('Switch to dark')
  })

  it('calls toggle when button is clicked', async () => {
    setupTheme(false)
    const w = mount(ThemeToggle)
    await w.find('button').trigger('click')
    expect(mockToggle).toHaveBeenCalledOnce()
  })
})

describe('ThemeToggle – dark mode', () => {
  it('renders sun icon when isDark is true', () => {
    setupTheme(true)
    const w = mount(ThemeToggle)
    expect(w.find('.theme-icon').html()).toContain('id="sun"')
  })

  it('aria-label and title reflect dark mode label', () => {
    setupTheme(true)
    const w = mount(ThemeToggle)
    expect(w.find('button').attributes('aria-label')).toBe('Switch to light')
    expect(w.find('button').attributes('title')).toBe('Switch to light')
  })
})

describe('ThemeToggle – structure & accessibility', () => {
  it('root element is a button', () => {
    setupTheme(false)
    const w = mount(ThemeToggle)
    expect(w.find('button').exists()).toBe(true)
  })

  it('button has type=button', () => {
    setupTheme(false)
    const w = mount(ThemeToggle)
    expect(w.find('button').attributes('type')).toBe('button')
  })

  it('button has theme-toggle class', () => {
    setupTheme(false)
    const w = mount(ThemeToggle)
    expect(w.find('button').classes()).toContain('theme-toggle')
  })

  it('icon span has aria-hidden=true', () => {
    setupTheme(false)
    const w = mount(ThemeToggle)
    expect(w.find('.theme-icon').attributes('aria-hidden')).toBe('true')
  })

  it('does NOT render moon icon in dark mode', () => {
    setupTheme(true)
    const w = mount(ThemeToggle)
    expect(w.find('.theme-icon').html()).not.toContain('id="moon"')
  })

  it('does NOT render sun icon in light mode', () => {
    setupTheme(false)
    const w = mount(ThemeToggle)
    expect(w.find('.theme-icon').html()).not.toContain('id="sun"')
  })

  it('calls toggle exactly once per click', async () => {
    setupTheme(true)
    const w = mount(ThemeToggle)
    await w.find('button').trigger('click')
    expect(mockToggle).toHaveBeenCalledTimes(1)
  })

  it('calls toggle twice on two clicks', async () => {
    setupTheme(false)
    const w = mount(ThemeToggle)
    await w.find('button').trigger('click')
    await w.find('button').trigger('click')
    expect(mockToggle).toHaveBeenCalledTimes(2)
  })
})
