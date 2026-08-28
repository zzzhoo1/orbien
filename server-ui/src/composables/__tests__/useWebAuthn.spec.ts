import {describe, it, expect, vi, beforeEach, afterEach} from 'vitest'
import {useWebAuthn} from '../useWebAuthn'

// ── helpers ────────────────────────────────────────────────────────────────────
function makeFakeCredential(extra: Record<string, unknown> = {}) {
  const data = new Uint8Array([1, 2, 3]).buffer
  return {
    id: 'fake-cred-id',
    rawId: data,
    type: 'public-key',
    response: {
      attestationObject: data,
      clientDataJSON: data,
      authenticatorData: data,
      signature: data,
      userHandle: null,
    },
    ...extra,
  }
}

function makeServerOptions(override: Record<string, unknown> = {}) {
  return {
    code: 0,
    msg: 'ok',
    data: {
      publicKey: {
        challenge: btoa('challenge-bytes'),
        timeout: 60000,
        user: {id: btoa('user-id'), name: 'alice', displayName: 'Alice'},
        rp: {id: 'example.com', name: 'Example'},
        pubKeyCredParams: [],
        excludeCredentials: [],
        allowCredentials: [],
        ...override,
      },
    },
  }
}

const fakeCredential = makeFakeCredential()

beforeEach(() => {
  Object.defineProperty(window, 'PublicKeyCredential', {
    value: class {},
    writable: true,
    configurable: true,
  })

  Object.defineProperty(navigator, 'credentials', {
    value: {
      create: vi.fn().mockResolvedValue(fakeCredential),
      get: vi.fn().mockResolvedValue(fakeCredential),
    },
    writable: true,
    configurable: true,
  })

  globalThis.fetch = vi.fn()
})

afterEach(() => {
  vi.restoreAllMocks()
})

// ── supported ref ─────────────────────────────────────────────────────────────────
describe('useWebAuthn – supported', () => {
  it('is true when PublicKeyCredential exists on window', () => {
    const {supported} = useWebAuthn()
    expect(supported.value).toBe(true)
  })

  it('is false when PublicKeyCredential is not defined', () => {
    // @ts-expect-error intentional
    delete window.PublicKeyCredential
    const {supported} = useWebAuthn()
    expect(supported.value).toBe(false)
  })
})

// ── register ───────────────────────────────────────────────────────────────────
describe('useWebAuthn – register', () => {
  it('throws WEBAUTHN_NOT_SUPPORTED when unsupported', async () => {
    // @ts-expect-error intentional
    delete window.PublicKeyCredential
    const {register} = useWebAuthn()
    await expect(register('alice')).rejects.toThrow('WEBAUTHN_NOT_SUPPORTED')
  })

  it('throws REGISTER_INIT_FAILED when begin endpoint returns non-2xx', async () => {
    ;(globalThis.fetch as ReturnType<typeof vi.fn>)
      .mockResolvedValueOnce({ok: false, json: vi.fn()})
    const {register} = useWebAuthn()
    await expect(register('alice')).rejects.toThrow('REGISTER_INIT_FAILED')
  })

  it('sets registering=true during register, false when done', async () => {
    let capturedDuring = false
    const {registering, register} = useWebAuthn()

    ;(globalThis.fetch as ReturnType<typeof vi.fn>)
      .mockImplementationOnce(async () => {
        capturedDuring = registering.value
        return {ok: true, json: async () => makeServerOptions()}
      })
      .mockResolvedValueOnce({ok: true})

    await register('alice')
    expect(capturedDuring).toBe(true)
    expect(registering.value).toBe(false)
  })

  it('resets registering=false even when register throws', async () => {
    ;(globalThis.fetch as ReturnType<typeof vi.fn>)
      .mockResolvedValueOnce({ok: false})
    const {registering, register} = useWebAuthn()
    await register('alice').catch(() => {})
    expect(registering.value).toBe(false)
  })

  it('calls navigator.credentials.create after begin', async () => {
    ;(globalThis.fetch as ReturnType<typeof vi.fn>)
      .mockResolvedValueOnce({ok: true, json: async () => makeServerOptions()})
      .mockResolvedValueOnce({ok: true})
    const {register} = useWebAuthn()
    await register('alice')
    expect(navigator.credentials.create).toHaveBeenCalledOnce()
  })

  it('throws REGISTER_FAILED when finish endpoint returns non-2xx', async () => {
    ;(globalThis.fetch as ReturnType<typeof vi.fn>)
      .mockResolvedValueOnce({ok: true, json: async () => makeServerOptions()})
      .mockResolvedValueOnce({ok: false})
    const {register} = useWebAuthn()
    await expect(register('alice')).rejects.toThrow('REGISTER_FAILED')
  })

  it('completes successfully when both begin and finish are ok', async () => {
    ;(globalThis.fetch as ReturnType<typeof vi.fn>)
      .mockResolvedValueOnce({ok: true, json: async () => makeServerOptions()})
      .mockResolvedValueOnce({ok: true})
    const {register} = useWebAuthn()
    await expect(register('alice')).resolves.toBeUndefined()
  })
})

// ── authenticate ───────────────────────────────────────────────────────────────
describe('useWebAuthn – authenticate', () => {
  it('throws WEBAUTHN_NOT_SUPPORTED when unsupported', async () => {
    // @ts-expect-error intentional
    delete window.PublicKeyCredential
    const {authenticate} = useWebAuthn()
    await expect(authenticate()).rejects.toThrow('WEBAUTHN_NOT_SUPPORTED')
  })

  it('throws WEBAUTHN_INIT_FAILED when begin endpoint returns non-2xx', async () => {
    ;(globalThis.fetch as ReturnType<typeof vi.fn>)
      .mockResolvedValueOnce({ok: false})
    const {authenticate} = useWebAuthn()
    await expect(authenticate()).rejects.toThrow('WEBAUTHN_INIT_FAILED')
  })

  it('sets authenticating=true during authenticate, false when done', async () => {
    let capturedDuring = false
    const {authenticating, authenticate} = useWebAuthn()

    ;(globalThis.fetch as ReturnType<typeof vi.fn>)
      .mockImplementationOnce(async () => {
        capturedDuring = authenticating.value
        return {ok: true, json: async () => makeServerOptions()}
      })
      .mockResolvedValueOnce({ok: true})

    await authenticate()
    expect(capturedDuring).toBe(true)
    expect(authenticating.value).toBe(false)
  })

  it('resets authenticating=false even when authenticate throws', async () => {
    ;(globalThis.fetch as ReturnType<typeof vi.fn>)
      .mockResolvedValueOnce({ok: false})
    const {authenticating, authenticate} = useWebAuthn()
    await authenticate().catch(() => {})
    expect(authenticating.value).toBe(false)
  })

  it('calls navigator.credentials.get after begin', async () => {
    ;(globalThis.fetch as ReturnType<typeof vi.fn>)
      .mockResolvedValueOnce({ok: true, json: async () => makeServerOptions()})
      .mockResolvedValueOnce({ok: true})
    const {authenticate} = useWebAuthn()
    await authenticate()
    expect(navigator.credentials.get).toHaveBeenCalledOnce()
  })

  it('returns true when finish endpoint is ok', async () => {
    ;(globalThis.fetch as ReturnType<typeof vi.fn>)
      .mockResolvedValueOnce({ok: true, json: async () => makeServerOptions()})
      .mockResolvedValueOnce({ok: true})
    const {authenticate} = useWebAuthn()
    expect(await authenticate()).toBe(true)
  })

  it('returns false when finish endpoint is not ok', async () => {
    ;(globalThis.fetch as ReturnType<typeof vi.fn>)
      .mockResolvedValueOnce({ok: true, json: async () => makeServerOptions()})
      .mockResolvedValueOnce({ok: false})
    const {authenticate} = useWebAuthn()
    expect(await authenticate()).toBe(false)
  })
})

// ── unwrapPublicKey ──────────────────────────────────────────────────────────────
describe('useWebAuthn – unwrapPublicKey', () => {
  it('throws UNEXPECTED_RESPONSE when body is a primitive (null)', async () => {
    ;(globalThis.fetch as ReturnType<typeof vi.fn>)
      .mockResolvedValueOnce({ok: true, json: async () => null})
    const {register} = useWebAuthn()
    await expect(register('alice')).rejects.toThrow('UNEXPECTED_RESPONSE')
  })
})
