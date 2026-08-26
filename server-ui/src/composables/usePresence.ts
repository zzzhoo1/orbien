import {useLocale} from '@/composables/useLocale'

export function usePresence() {
    const {t} = useLocale()

    function isOnline(raw?: string) {
        return !raw || raw === 'online'
    }

    function statusLabel(raw?: string) {
        if (isOnline(raw)) return t('status.online')
        if (raw === 'offline') return t('status.offline')
        return raw || t('status.offline')
    }

    function formatSeen(secs: number, online: boolean) {
        const n = Math.max(0, Math.floor(secs || 0))
        if (online) {
            if (n < 60) return t('clients.uptimeSecs', {n})
            if (n < 3600) return t('clients.uptimeMins', {n: Math.floor(n / 60)})
            if (n < 86400) return t('clients.uptimeHours', {n: Math.floor(n / 3600)})
            return t('clients.uptimeDays', {n: Math.floor(n / 86400)})
        }
        if (n < 60) return t('clients.agoSecs', {n})
        if (n < 3600) return t('clients.agoMins', {n: Math.floor(n / 60)})
        if (n < 86400) return t('clients.agoHours', {n: Math.floor(n / 3600)})
        return t('clients.agoDays', {n: Math.floor(n / 86400)})
    }

    return {isOnline, statusLabel, formatSeen}
}
