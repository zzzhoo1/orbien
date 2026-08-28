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
