import {describe, it, expect} from 'vitest'
import {ApiError, isApiError} from '../errors'

describe('ApiError', () => {
  it('is an instance of Error', () => {
    expect(new ApiError('unknown')).toBeInstanceOf(Error)
  })

  it('sets name to ApiError', () => {
    expect(new ApiError('unauthorized').name).toBe('ApiError')
  })

  it('sets message to code string', () => {
    expect(new ApiError('http').message).toBe('http')
  })

  it('stores the code correctly', () => {
    const err = new ApiError('api')
    expect(err.code).toBe('api')
  })

  it('stores params when provided', () => {
    const err = new ApiError('http', {status: 404})
    expect(err.params).toEqual({status: 404})
  })

  it('params is undefined when not provided', () => {
    const err = new ApiError('unknown')
    expect(err.params).toBeUndefined()
  })

  it('stores cause when provided', () => {
    const cause = new Error('original')
    const err = new ApiError('unknown', undefined, cause)
    expect((err as unknown as {cause: unknown}).cause).toBe(cause)
  })

  it('does not set cause when undefined', () => {
    const err = new ApiError('unknown')
    expect(Object.prototype.hasOwnProperty.call(err, 'cause')).toBe(false)
  })

  it('supports all valid code values', () => {
    const codes = ['unauthorized', 'http', 'api', 'unknown'] as const
    for (const code of codes) {
      expect(new ApiError(code).code).toBe(code)
    }
  })

  it('has a stack trace', () => {
    const err = new ApiError('unknown')
    expect(err.stack).toBeTruthy()
    expect(typeof err.stack).toBe('string')
  })

  it('stores params with multiple fields', () => {
    const err = new ApiError('http', {status: 503, detail: 'unavailable'})
    expect(err.params).toEqual({status: 503, detail: 'unavailable'})
  })

  it('params undefined when explicitly passed undefined', () => {
    const err = new ApiError('api', undefined)
    expect(err.params).toBeUndefined()
  })

  it('two ApiErrors with same code are independent instances', () => {
    const a = new ApiError('http', {status: 400})
    const b = new ApiError('http', {status: 500})
    expect(a.params?.status).toBe(400)
    expect(b.params?.status).toBe(500)
  })

  it('code=unauthorized message equals "unauthorized"', () => {
    expect(new ApiError('unauthorized').message).toBe('unauthorized')
  })

  it('code=unknown message equals "unknown"', () => {
    expect(new ApiError('unknown').message).toBe('unknown')
  })
})

describe('isApiError', () => {
  it('returns true for ApiError instances', () => {
    expect(isApiError(new ApiError('unknown'))).toBe(true)
  })

  it('returns true for ApiError with params', () => {
    expect(isApiError(new ApiError('http', {status: 404}))).toBe(true)
  })

  it('returns true for ApiError with cause', () => {
    expect(isApiError(new ApiError('api', undefined, new Error('net')))).toBe(true)
  })

  it('returns false for plain Error instances', () => {
    expect(isApiError(new Error('oops'))).toBe(false)
  })

  it('returns false for null', () => {
    expect(isApiError(null)).toBe(false)
  })

  it('returns false for undefined', () => {
    expect(isApiError(undefined)).toBe(false)
  })

  it('returns false for strings', () => {
    expect(isApiError('error')).toBe(false)
  })

  it('returns false for numbers', () => {
    expect(isApiError(42)).toBe(false)
  })

  it('returns false for plain objects', () => {
    expect(isApiError({code: 'http'})).toBe(false)
  })

  it('returns false for arrays', () => {
    expect(isApiError(['http'])).toBe(false)
  })

  it('returns false for boolean', () => {
    expect(isApiError(true)).toBe(false)
  })

  it('correctly distinguishes between ApiError and subclass of Error', () => {
    class OtherError extends Error {}
    expect(isApiError(new OtherError('test'))).toBe(false)
  })
})
