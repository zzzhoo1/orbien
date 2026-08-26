import {computed} from 'vue'
import {useI18n} from 'vue-i18n'
import {
    LOCALE_META,
    SUPPORTED_LOCALES,
    setLocale,
    type AppLocale,
    type MessageSchema,
} from '@/i18n'

export function useLocale() {
    const {locale, t} = useI18n<{message: MessageSchema}, AppLocale>()

    const current = computed(() => locale.value as AppLocale)
    const options = SUPPORTED_LOCALES.map((code) => ({
        code,
        ...LOCALE_META[code],
    }))

    function switchLocale(next: AppLocale) {
        setLocale(next)
    }

    return {t, current, options, switchLocale}
}
