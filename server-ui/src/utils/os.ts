export type OsFamily = 'windows' | 'macos' | 'linux' | 'android' | 'freebsd' | 'other'

export function normalizeOsFamily(raw?: string | null): OsFamily {
    const s = (raw || '').trim().toLowerCase()
    if (!s) return 'other'
    if (s.includes('win')) return 'windows'
    if (s.includes('mac') || s.includes('darwin') || s === 'osx') return 'macos'
    if (s.includes('android')) return 'android'
    if (s.includes('freebsd')) return 'freebsd'
    if (s.includes('linux') || s.includes('ubuntu') || s.includes('debian') || s.includes('centos')) {
        return 'linux'
    }
    return 'other'
}

export function formatArch(raw?: string | null): string {
    const s = (raw || '').trim().toLowerCase()
    if (!s) return ''
    if (s === 'aarch64' || s === 'arm64') return 'arm64'
    if (s === 'x86_64' || s === 'amd64' || s === 'x64') return 'x64'
    if (s === 'i386' || s === 'i686' || s === 'x86') return 'x86'
    return raw!.trim()
}
