package io.github.lxien.orbien.boot;

import io.github.lxien.orbien.client.OrbienClient;
import io.github.lxien.orbien.client.OrbienClientConfig;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.boot.context.event.ApplicationReadyEvent;
import org.springframework.boot.web.context.WebServerApplicationContext;
import org.springframework.context.ApplicationContext;
import org.springframework.context.event.EventListener;
import org.springframework.core.env.Environment;
import org.springframework.util.StringUtils;

public class OrbienClientLifecycle {
    private static final Logger log = LoggerFactory.getLogger(OrbienClientLifecycle.class);
    private static final String DEFAULT_LOCAL_IP = "127.0.0.1";

    private final OrbienClient client;
    private final OrbienProperties properties;

    public OrbienClientLifecycle(OrbienClient client, OrbienProperties properties) {
        this.client = client;
        this.properties = properties;
    }

    @EventListener(ApplicationReadyEvent.class)
    public void onReady(ApplicationReadyEvent event) {
        if (!properties.isEnabled()) {
            return;
        }
        if (properties.isTcpMux()) {
            throw new IllegalStateException(
                    "orbien.tcp-mux=true is not supported; set false on client and server");
        }
        applyLocalDefaults(event.getApplicationContext());
        log.info("starting Orbien client");
        client.start();
    }

    private void applyLocalDefaults(ApplicationContext applicationContext) {
        OrbienProperties.Tunnel tunnelProps = properties.getTunnel();
        if (!properties.hasTunnel()) {
            return;
        }

        String localIp = tunnelProps.getLocalIp();
        if (!StringUtils.hasText(localIp)) {
            localIp = DEFAULT_LOCAL_IP;
            tunnelProps.setLocalIp(localIp);
        }

        int localPort = tunnelProps.getLocalPort();
        if (localPort <= 0) {
            localPort = resolveLocalPort(applicationContext);
            tunnelProps.setLocalPort(localPort);
            log.info("orbien.tunnel.local-port not set; using Spring Boot port {}", localPort);
        }

        String name = tunnelProps.getName();
        if (!StringUtils.hasText(name)) {
            name = defaultTunnelName(applicationContext.getEnvironment(), tunnelProps.getProtocol());
            tunnelProps.setName(name);
            log.info("orbien.tunnel.name not set; using {}", name);
        }

        syncClientTunnel(name, localIp, localPort);
    }

    private void syncClientTunnel(String name, String localIp, int localPort) {
        OrbienClientConfig cfg = client.config();
        OrbienClientConfig.TunnelConfig tunnel;
        if (cfg.getTunnels().isEmpty()) {
            tunnel = properties.toClientConfig().getTunnels().stream().findFirst().orElse(null);
            if (tunnel == null) {
                return;
            }
            cfg.getTunnels().add(tunnel);
        } else {
            tunnel = cfg.getTunnels().get(0);
        }
        tunnel.setName(name);
        tunnel.setLocalIp(localIp);
        tunnel.setLocalPort(localPort);
    }

    static int resolveLocalPort(ApplicationContext applicationContext) {
        if (applicationContext instanceof WebServerApplicationContext webContext) {
            try {
                int port = webContext.getWebServer().getPort();
                if (port > 0) {
                    return port;
                }
            } catch (IllegalStateException ex) {
                log.debug("web server not ready while resolving local port: {}", ex.getMessage());
            }
        }

        Environment env = applicationContext.getEnvironment();
        Integer localServerPort = env.getProperty("local.server.port", Integer.class);
        if (localServerPort != null && localServerPort > 0) {
            return localServerPort;
        }

        Integer serverPort = env.getProperty("server.port", Integer.class);
        if (serverPort != null && serverPort > 0) {
            return serverPort;
        }

        throw new IllegalStateException(
                "orbien.tunnel.local-port is not set and the Spring Boot web server port could not be"
                        + " determined; set orbien.tunnel.local-port explicitly");
    }

    static String defaultTunnelName(Environment env, String type) {
        String appName = env.getProperty("spring.application.name");
        if (StringUtils.hasText(appName)) {
            return appName.trim();
        }
        String tunnelType = StringUtils.hasText(type) ? type.trim().toLowerCase() : "tcp";
        return "orbien-" + tunnelType;
    }
}
