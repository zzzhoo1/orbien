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

/** Known values: 'online' | 'offline'. The union with `string & {}` keeps
 *  the type open for future server additions while surfacing completions. */
export type PresenceStatus = 'online' | 'offline' | (string & {})

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
    status: PresenceStatus
    totalTrafficIn?: number
    totalTrafficOut?: number
}

export interface TunnelInfo {
    name: string
    type: string
    remoteAddr: string
    localAddr: string
    sessionId: string
    status: PresenceStatus
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

/** A single active connection through a tunnel. */
export interface ConnectionInfo {
    /** Unique connection identifier (may be a number or opaque string). */
    id: string | number
    /** Remote visitor address, e.g. "1.2.3.4:54321". */
    remoteAddr: string
    /** Local target address, e.g. "127.0.0.1:3000". */
    localAddr?: string
    /** ISO-8601 timestamp when the connection was established. */
    connectedAt?: string
    /** Bytes received from the visitor side. */
    trafficIn?: number
    /** Bytes sent to the visitor side. */
    trafficOut?: number
    /** Connection status, typically 'active'. */
    status?: string
}

/**
 * Response from `POST /api/v1/config/reload`.
 * `changed` lists top-level config keys that differed from the previous load.
 * An empty array means the file was re-read successfully with no observable changes.
 */
export interface ConfigReloadResp {
    changed: string[]
}

/**
 * Response from `GET /api/v1/system/health`.
 * Used by the Settings page health card and any external health polling.
 */
export interface HealthInfo {
    status: 'ok' | string
    uptimeSecs: number
    version: string
    onlineClients: number
    activeConnections: number
}
