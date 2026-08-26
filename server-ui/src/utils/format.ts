export function formatFileSize(bytes: number | null | undefined): string {
    const n = Number(bytes ?? 0)
    if (!Number.isFinite(n) || n <= 0) return '0 B'
    const units = ['B', 'KB', 'MB', 'GB', 'TB']
    let v = n
    let i = 0
    while (v >= 1024 && i < units.length - 1) {
        v /= 1024
        i += 1
    }
    const digits = i === 0 ? 0 : v < 10 ? 2 : v < 100 ? 1 : 0
    return `${v.toFixed(digits)} ${units[i]}`
}

export function isUnsetPort(value: number | null | undefined): boolean {
    return value == null || value === 0
}

export function isUnsetText(value: string | null | undefined): boolean {
    return value == null || value.trim() === ''
}

export function formatPort(value: number | null | undefined): string | null {
    if (isUnsetPort(value)) return null
    return String(value)
}

export function formatText(value: string | null | undefined): string | null {
    if (isUnsetText(value)) return null
    return value!.trim()
}

export function isHttpTunnelType(type?: string | null): boolean {
    const t = (type || '').toLowerCase()
    return t === 'http' || t === 'https'
}

export function formatTunnelEndpoint(
    type?: string | null,
    remoteAddr?: string | null,
): string {
    const raw = (remoteAddr || '').trim()
    if (!raw) return '—'
    if (isHttpTunnelType(type)) return raw
    return raw.replace(/^:/, '') || '—'
}
