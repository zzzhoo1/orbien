import {describe, it, expect, vi, beforeEach, afterEach} from 'vitest'
import {mount} from '@vue/test-utils'
import {defineComponent} from 'vue'
import {useToast} from '../useToast'

function mountToast() {
  let result: ReturnType<typeof useToast>
  mount(defineComponent({
    setup() { result = useToast(); return {} },
    template: '<div/>',
  }))
  return result!
}

describe('useToast', () => {
  beforeEach(() => {
    vi.useFakeTimers()
    const {show} = mountToast()
    show('info', '__reset__', 0)
    vi.advanceTimersByTime(1)
  })

  afterEach(() => {
    vi.useRealTimers()
  })

  it('message is null after reset', () => {
    const {message} = mountToast()
    expect(message.value).toBeNull()
  })

  it('show sets message immediately', () => {
    const {show, message} = mountToast()
    show('info', 'Hello')
    expect(message.value).not.toBeNull()
    expect(message.value!.text).toBe('Hello')
    expect(message.value!.type).toBe('info')
  })

  it('show with type error sets type correctly', () => {
    const {show, message} = mountToast()
    show('error', 'Oops')
    expect(message.value!.type).toBe('error')
  })

  it('message is cleared after default duration 3000ms', () => {
    const {show, message} = mountToast()
    show('info', 'temp')
    expect(message.value).not.toBeNull()
    vi.advanceTimersByTime(3000)
    expect(message.value).toBeNull()
  })

  it('message is cleared after custom duration', () => {
    const {show, message} = mountToast()
    show('info', 'short', 500)
    vi.advanceTimersByTime(499)
    expect(message.value).not.toBeNull()
    vi.advanceTimersByTime(1)
    expect(message.value).toBeNull()
  })

  it('message is not cleared before default duration elapses', () => {
    const {show, message} = mountToast()
    show('info', 'persistent')
    vi.advanceTimersByTime(2999)
    expect(message.value).not.toBeNull()
  })

  it('calling show twice resets the timer', () => {
    const {show, message} = mountToast()
    show('info', 'first', 3000)
    vi.advanceTimersByTime(2000)
    show('info', 'second', 3000)
    vi.advanceTimersByTime(2000)
    expect(message.value).not.toBeNull()
    expect(message.value!.text).toBe('second')
    vi.advanceTimersByTime(1000)
    expect(message.value).toBeNull()
  })

  it('each show increments id', () => {
    const {show, message} = mountToast()
    show('info', 'a')
    const id1 = message.value!.id
    show('info', 'b')
    const id2 = message.value!.id
    expect(id2).toBeGreaterThan(id1)
  })

  it('message has correct text field', () => {
    const {show, message} = mountToast()
    show('error', 'something went wrong')
    expect(message.value!.text).toBe('something went wrong')
  })

  it('duration=0 clears message immediately', () => {
    const {show, message} = mountToast()
    show('info', 'instant', 0)
    vi.advanceTimersByTime(0)
    expect(message.value).toBeNull()
  })

  it('message id is a positive number', () => {
    const {show, message} = mountToast()
    show('info', 'test')
    expect(message.value!.id).toBeGreaterThan(0)
  })
})
