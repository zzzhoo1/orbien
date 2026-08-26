package io.github.lxien.orbien.client;

import java.util.ArrayList;
import java.util.List;
import java.util.Objects;

public final class OrbienClientConfig {
    private String serverAddr = "127.0.0.1";
    private int serverPort = 9527;
    private String token = "";
    private boolean tcpMux = false;
    private int poolCount = 1;
    private String user = "";
    private String sessionId = "";
    private String sessionIdFile = "";
    private int heartbeatIntervalSecs = 30;
    private final List<TunnelConfig> tunnels = new ArrayList<>();

    public String getServerAddr() {
        return serverAddr;
    }

    public void setServerAddr(String serverAddr) {
        this.serverAddr = Objects.requireNonNull(serverAddr, "serverAddr");
    }

    public int getServerPort() {
        return serverPort;
    }

    public void setServerPort(int serverPort) {
        this.serverPort = serverPort;
    }

    public String getToken() {
        return token;
    }

    public void setToken(String token) {
        this.token = token == null ? "" : token;
    }

    public boolean isTcpMux() {
        return tcpMux;
    }

    public void setTcpMux(boolean tcpMux) {
        this.tcpMux = tcpMux;
    }

    public int getPoolCount() {
        return poolCount;
    }

    public void setPoolCount(int poolCount) {
        this.poolCount = poolCount;
    }

    public String getUser() {
        return user;
    }

    public void setUser(String user) {
        this.user = user == null ? "" : user;
    }

    public String getSessionId() {
        return sessionId;
    }

    public void setSessionId(String sessionId) {
        this.sessionId = sessionId == null ? "" : sessionId;
    }

    public String getSessionIdFile() {
        return sessionIdFile;
    }

    public void setSessionIdFile(String sessionIdFile) {
        this.sessionIdFile = sessionIdFile == null ? "" : sessionIdFile;
    }

    public int getHeartbeatIntervalSecs() {
        return heartbeatIntervalSecs;
    }

    public void setHeartbeatIntervalSecs(int heartbeatIntervalSecs) {
        this.heartbeatIntervalSecs = heartbeatIntervalSecs;
    }

    public List<TunnelConfig> getTunnels() {
        return tunnels;
    }

    public static final class TunnelConfig {
        private String protocol = "tcp";
        private String name;
        private String localIp = "127.0.0.1";
        private int localPort;
        private int remotePort;
        private List<String> domains = new ArrayList<>();

        public String getProtocol() {
            return protocol;
        }

        public void setProtocol(String protocol) {
            this.protocol = protocol;
        }

        public String getName() {
            return name;
        }

        public void setName(String name) {
            this.name = name;
        }

        public String getLocalIp() {
            return localIp;
        }

        public void setLocalIp(String localIp) {
            this.localIp = localIp;
        }

        public int getLocalPort() {
            return localPort;
        }

        public void setLocalPort(int localPort) {
            this.localPort = localPort;
        }

        public int getRemotePort() {
            return remotePort;
        }

        public void setRemotePort(int remotePort) {
            this.remotePort = remotePort;
        }

        public List<String> getDomains() {
            return domains;
        }

        public void setDomains(List<String> domains) {
            this.domains = domains == null ? new ArrayList<>() : domains;
        }
    }
}
