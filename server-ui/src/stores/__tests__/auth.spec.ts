import {describe, it, expect, vi, beforeEach} from 'vitest'
import {setActivePinia, createPinia} from 'pinia'
import {useAuthStore} from '../auth'

const mockFetchAuthStatus = vi.fn()
vi.mock('@/api/client', () => ({
  fetchAuthStatus: (...args: unknown[]) => mockFetchAuthStatus(...args),
}))

beforeEach(() => {
  setActivePinia(createPinia())
  vi.clearAllMocks()
  globalThis.fetch = vi.fn()
})

describe('useAuthStore – initial state', () => {
  it('authenticated is false', () => {
    expect(useAuthStore().authenticated).toBe(false)
  })
  it('username is empty string', () => {
    expect(useAuthStore().username).toBe('')
  })
  it('capabilitiesLoaded is false', () => {
    expect(useAuthStore().capabilitiesLoaded).toBe(false)
  })
  it('capabilities defaults to password=true, webauthn=false', () => {
    expect(useAuthStore().capabilities).toEqual({webauthn: false, password: true})
  })
})

describe('useAuthStore – setAuthenticated', () => {
  it('sets authenticated and username', () => {
    const store = useAuthStore()
    store.setAuthenticated(true, 'alice')
    expect(store.authenticated).toBe(true)
    expect(store.username).toBe('alice')
  })
  it('clears authenticated and username', () => {
    const store = useAuthStore()
    store.setAuthenticated(true, 'bob')
    store.setAuthenticated(false)
    expect(store.authenticated).toBe(false)
    expect(store.username).toBe('')
  })
})

describe('useAuthStore – loadCapabilities', () => {
  it('calls fetchAuthStatus and stores result', async () => {
    mockFetchAuthStatus.mockResolvedValue({webauthn: true, password: false})
    const store = useAuthStore()
    await store.loadCapabilities()
    expect(mockFetchAuthStatus).toHaveBeenCalledOnce()
    expect(store.capabilities).toEqual({webauthn: true, password: false})
    expect(store.capabilitiesLoaded).toBe(true)
  })

  it('does NOT call fetchAuthStatus a second time once loaded', async () => {
    mockFetchAuthStatus.mockResolvedValue({webauthn: false, password: true})
    const store = useAuthStore()
    await store.loadCapabilities()
    await store.loadCapabilities()
    expect(mockFetchAuthStatus).toHaveBeenCalledOnce()
  })
})

describe('useAuthStore – fetchStatus', () => {
  it('sets authenticated=true when response is ok', async () => {
    ;(globalThis.fetch as ReturnType<typeof vi.fn>).mockResolvedValue({ok: true})
    const store = useAuthStore()
    const result = await store.fetchStatus()
    expect(result).toBe(true)
    expect(store.authenticated).toBe(true)
  })

  it('sets authenticated=false when response is not ok', async () => {
    ;(globalThis.fetch as ReturnType<typeof vi.fn>).mockResolvedValue({ok: false})
    const store = useAuthStore()
    const result = await store.fetchStatus()
    expect(result).toBe(false)
    expect(store.authenticated).toBe(false)
  })

  it('sets authenticated=false when fetch throws', async () => {
    ;(globalThis.fetch as ReturnType<typeof vi.fn>).mockRejectedValue(new Error('network'))
    const store = useAuthStore()
    const result = await store.fetchStatus()
    expect(result).toBe(false)
    expect(store.authenticated).toBe(false)
  })
})

describe('useAuthStore – loginWithPassword', () => {
  it('sets authenticated=true on success', async () => {
    ;(globalThis.fetch as ReturnType<typeof vi.fn>).mockResolvedValue({ok: true, status: 200})
    const store = useAuthStore()
    await store.loginWithPassword('alice', 'pass')
    expect(store.authenticated).toBe(true)
    expect(store.username).toBe('alice')
  })

  it('throws UNAUTHORIZED on 401', async () => {
    ;(globalThis.fetch as ReturnType<typeof vi.fn>).mockResolvedValue({ok: false, status: 401})
    const store = useAuthStore()
    await expect(store.loginWithPassword('alice', 'wrong')).rejects.toThrow('UNAUTHORIZED')
    expect(store.authenticated).toBe(false)
  })

  it('throws HTTP_XXX on other error statuses', async () => {
    ;(globalThis.fetch as ReturnType<typeof vi.fn>).mockResolvedValue({ok: false, status: 500})
    const store = useAuthStore()
    await expect(store.loginWithPassword('alice', 'pass')).rejects.toThrow('HTTP_500')
  })
})

describe('useAuthStore – logout', () => {
  it('clears authenticated and username', async () => {
    ;(globalThis.fetch as ReturnType<typeof vi.fn>).mockResolvedValue({ok: true})
    const store = useAuthStore()
    store.setAuthenticated(true, 'alice')
    await store.logout()
    expect(store.authenticated).toBe(false)
    expect(store.username).toBe('')
  })

  it('resets capabilitiesLoaded so next login page re-fetches', async () => {
    ;(globalThis.fetch as ReturnType<typeof vi.fn>).mockResolvedValue({ok: true})
    mockFetchAuthStatus.mockResolvedValue({webauthn: false, password: true})
    const store = useAuthStore()
    await store.loadCapabilities()
    expect(store.capabilitiesLoaded).toBe(true)
    await store.logout()
    expect(store.capabilitiesLoaded).toBe(false)
  })

  it('does not throw if logout fetch fails', async () => {
    ;(globalThis.fetch as ReturnType<typeof vi.fn>).mockRejectedValue(new Error('net'))
    const store = useAuthStore()
    await expect(store.logout()).resolves.toBeUndefined()
  })
})
