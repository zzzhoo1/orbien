import {describe, expect, it} from 'vitest'
import {mount} from '@vue/test-utils'
import EmptyState from '../EmptyState.vue'

// ── helpers ───────────────────────────────────────────────────────────────────

function mountEmptyState(props: Record<string, unknown>) {
  return mount(EmptyState, {props})
}

// ── suite ─────────────────────────────────────────────────────────────────────

describe('EmptyState', () => {
  // ── required props ──────────────────────────────────────────────────────────

  it('renders title', () => {
    const wrapper = mountEmptyState({title: 'No clients yet'})
    expect(wrapper.find('.empty-title').text()).toBe('No clients yet')
  })

  it('does not render desc when omitted', () => {
    const wrapper = mountEmptyState({title: 'No clients yet'})
    expect(wrapper.find('.empty-desc').exists()).toBe(false)
  })

  it('renders desc when provided', () => {
    const wrapper = mountEmptyState({
      title: 'No clients yet',
      desc: 'Connect your first client to get started.',
    })
    expect(wrapper.find('.empty-desc').text()).toBe(
      'Connect your first client to get started.',
    )
  })

  // ── accessibility ────────────────────────────────────────────────────────────

  it('has role="status" on root element', () => {
    const wrapper = mountEmptyState({title: 'Nothing here'})
    expect(wrapper.find('.empty-state').attributes('role')).toBe('status')
  })

  it('has aria-live="polite" on root element', () => {
    const wrapper = mountEmptyState({title: 'Nothing here'})
    expect(wrapper.find('.empty-state').attributes('aria-live')).toBe('polite')
  })

  it('illustration wrapper has aria-hidden="true"', () => {
    const wrapper = mountEmptyState({title: 'Nothing here'})
    expect(wrapper.find('.empty-illustration').attributes('aria-hidden')).toBe('true')
  })

  // ── type variants ────────────────────────────────────────────────────────────

  describe('type="clients"', () => {
    it('renders only the clients SVG', () => {
      const wrapper = mountEmptyState({type: 'clients', title: 'No clients'})
      const svgs = wrapper.findAll('.empty-illustration svg')
      // v-if chain: only one SVG should be in the DOM
      expect(svgs).toHaveLength(1)
      // The clients SVG has two <circle> elements for avatars
      const circles = svgs[0].findAll('circle')
      expect(circles.length).toBeGreaterThanOrEqual(2)
    })
  })

  describe('type="tunnels"', () => {
    it('renders only the tunnels SVG', () => {
      const wrapper = mountEmptyState({type: 'tunnels', title: 'No tunnels'})
      const svgs = wrapper.findAll('.empty-illustration svg')
      expect(svgs).toHaveLength(1)
      // tunnels SVG has a dashed <path> for the connection line
      const paths = svgs[0].findAll('path')
      expect(paths.length).toBeGreaterThan(0)
    })
  })

  describe('type="search"', () => {
    it('renders only the search SVG', () => {
      const wrapper = mountEmptyState({type: 'search', title: 'No results'})
      const svgs = wrapper.findAll('.empty-illustration svg')
      expect(svgs).toHaveLength(1)
      // search SVG has a <circle> for the magnifier lens
      const circles = svgs[0].findAll('circle')
      expect(circles.length).toBeGreaterThanOrEqual(1)
    })
  })

  describe('type="filter"', () => {
    it('renders only the filter SVG', () => {
      const wrapper = mountEmptyState({type: 'filter', title: 'No matches'})
      const svgs = wrapper.findAll('.empty-illustration svg')
      expect(svgs).toHaveLength(1)
      // filter SVG has a funnel <path>
      const paths = svgs[0].findAll('path')
      expect(paths.length).toBeGreaterThan(0)
    })
  })

  describe('type="generic" (default / explicit)', () => {
    it('renders generic SVG when type is omitted', () => {
      const wrapper = mountEmptyState({title: 'Nothing'})
      const svgs = wrapper.findAll('.empty-illustration svg')
      expect(svgs).toHaveLength(1)
    })

    it('renders generic SVG when type="generic"', () => {
      const wrapper = mountEmptyState({type: 'generic', title: 'Nothing'})
      const svgs = wrapper.findAll('.empty-illustration svg')
      expect(svgs).toHaveLength(1)
    })
  })

  // ── full-render structural test (replaces fragile html snapshot) ──────────

  it('matches snapshot with all props', () => {
    const wrapper = mountEmptyState({
      type: 'clients',
      title: 'No clients yet',
      desc: 'Connect a client to begin.',
    })

    // root structure
    const root = wrapper.find('.empty-state')
    expect(root.exists()).toBe(true)
    expect(root.attributes('role')).toBe('status')
    expect(root.attributes('aria-live')).toBe('polite')

    // illustration block: exactly one SVG
    const illustration = wrapper.find('.empty-illustration')
    expect(illustration.exists()).toBe(true)
    expect(illustration.attributes('aria-hidden')).toBe('true')
    expect(illustration.findAll('svg')).toHaveLength(1)

    // clients SVG: has circles for avatar shapes
    const circles = illustration.findAll('circle')
    expect(circles.length).toBeGreaterThanOrEqual(2)

    // text content
    expect(wrapper.find('.empty-title').text()).toBe('No clients yet')
    expect(wrapper.find('.empty-desc').text()).toBe('Connect a client to begin.')
  })
})
