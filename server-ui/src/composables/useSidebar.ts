import {computed, ref, watch} from 'vue'

const STORAGE_KEY = 'orbien-server-ui-sidebar-collapsed'
const MOBILE_MQ = '(max-width: 900px)'

const collapsed = ref(false)
const mobileOpen = ref(false)
const isMobile = ref(false)

let started = false

function syncMobile() {
    isMobile.value = window.matchMedia(MOBILE_MQ).matches
    if (!isMobile.value) {
        mobileOpen.value = false
    }
}

function ensureListeners() {
    if (started || typeof window === 'undefined') return
    started = true
    collapsed.value = localStorage.getItem(STORAGE_KEY) === '1'
    syncMobile()
    window.matchMedia(MOBILE_MQ).addEventListener('change', syncMobile)
}

export function useSidebar() {
    ensureListeners()

    const desktopCollapsed = computed(() => !isMobile.value && collapsed.value)

    function toggleCollapsed() {
        if (isMobile.value) {
            mobileOpen.value = !mobileOpen.value
            return
        }
        collapsed.value = !collapsed.value
        localStorage.setItem(STORAGE_KEY, collapsed.value ? '1' : '0')
    }

    function closeMobile() {
        mobileOpen.value = false
    }

    watch([mobileOpen, isMobile], ([open, mobile]) => {
        document.body.style.overflow = open && mobile ? 'hidden' : ''
    })

    return {
        collapsed,
        mobileOpen,
        isMobile,
        desktopCollapsed,
        toggleCollapsed,
        closeMobile,
    }
}
