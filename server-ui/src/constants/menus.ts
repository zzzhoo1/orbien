export type NavSection = 'dashboard' | 'clients' | 'tunnels' | 'tokens' | 'settings'

export interface NavItem {
    key: NavSection
    labelKey: string
    path: string
    icon: string
}

export const NAV_ITEMS: NavItem[] = [
    { key: 'dashboard', labelKey: 'nav.dashboard', path: '/',         icon: 'dashboard' },
    { key: 'clients',   labelKey: 'nav.clients',   path: '/clients',  icon: 'clients'   },
    { key: 'tunnels',   labelKey: 'nav.tunnels',   path: '/tunnels',  icon: 'tunnels'   },
    { key: 'tokens',    labelKey: 'nav.tokens',    path: '/tokens',   icon: 'tokens'    },
    { key: 'settings',  labelKey: 'nav.settings',  path: '/settings', icon: 'settings'  },
]
