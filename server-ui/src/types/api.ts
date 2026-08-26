export interface ApiResponse<T> {
    code: number
    msg: string
    data: T
}

export interface Page<T> {
    total: number
    page: number
    pageSize: number
    items: T[]
}

export interface SystemInfo {
    version: string
    config: SystemConfig
    status: SystemStatus
}

export interface SystemConfig {
    listen: string
    quicPort: number
    kcpPort: number
    httpGwPort: number
    httpsGwPort: number
    rootDomain: string
    tcpMux: boolean
    tlsForce: boolean
    maxConnPool: number
    heartbeatTimeout: number
}

export interface SystemStatus {
    clientCounts: number
    totalClientCounts: number
    tunnelTypeCount: Record<string, number>
    activeConnections: number
    totalTrafficIn: number
    totalTrafficOut: number
}

export interface ClientInfo {
    sessionId: string
    user: string
    hostname: string
    os: string
    arch: string
    clientIP: string
    version: string
    tunnelCount: number
    activeConnections: number
    connectedSecs: number
    status: string
    totalTrafficIn?: number
    totalTrafficOut?: number
}

export interface TunnelInfo {
    name: string
    type: string
    remoteAddr: string
    localAddr: string
    sessionId: string
    status: string
    todayTrafficIn: number
    todayTrafficOut: number
    activeConnections: number
    lastStartTime?: string
    historyConns?: number
}

export interface TunnelTrafficPoint {
    date: string
    trafficIn: number
    trafficOut: number
}

export interface TunnelTrafficResp {
    name: string
    unit: 'bytes' | string
    granularity: 'day' | string
    history: TunnelTrafficPoint[]
}

export interface TokenMetricItem {
    token: string
    activeConns: number
    allowedTunnels: string[]
    allowedProtocols: string[]
    allowedRemotePorts: number[]
}

export interface TokenMetricsResp {
    tokens: TokenMetricItem[]
}
