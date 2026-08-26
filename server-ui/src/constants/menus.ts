export interface NavItem {
    name: 'monitor' | 'tunnels' | 'clients'
    path: string
    labelKey: 'monitor' | 'tunnels' | 'clients'
    icon: 'monitor' | 'tunnels' | 'clients'
}

export const NAV_ITEMS: readonly NavItem[] = [
    {name: 'monitor', path: '/', labelKey: 'monitor', icon: 'monitor'},
    {name: 'tunnels', path: '/tunnels', labelKey: 'tunnels', icon: 'tunnels'},
    {name: 'clients', path: '/clients', labelKey: 'clients', icon: 'clients'},
]
