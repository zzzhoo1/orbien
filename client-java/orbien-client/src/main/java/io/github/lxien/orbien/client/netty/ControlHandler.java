package io.github.lxien.orbien.client.netty;

import io.github.lxien.orbien.client.OrbienClientConfig;
import io.github.lxien.orbien.client.auth.AuthKeys;
import io.github.lxien.orbien.client.msg.KickOut;
import io.github.lxien.orbien.client.msg.LoginResp;
import io.github.lxien.orbien.client.msg.MsgType;
import io.github.lxien.orbien.client.msg.NewTunnel;
import io.github.lxien.orbien.client.msg.NewTunnelResp;
import io.github.lxien.orbien.client.msg.Ping;
import io.github.lxien.orbien.client.msg.WireMessage;
import io.netty.channel.ChannelHandlerContext;
import io.netty.channel.SimpleChannelInboundHandler;
import io.netty.util.concurrent.ScheduledFuture;

import java.util.concurrent.TimeUnit;
import java.util.concurrent.atomic.AtomicBoolean;
import java.util.function.Consumer;

import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

public final class ControlHandler extends SimpleChannelInboundHandler<WireMessage> {
    private static final Logger log = LoggerFactory.getLogger(ControlHandler.class);

    private final OrbienClientConfig config;
    private final Consumer<String> onLoginOk;
    private final Consumer<Throwable> onFailure;
    private final DataConnFactory dataFactory;
    private final AtomicBoolean tunnelsRegistered = new AtomicBoolean(false);
    private volatile String sessionId = "";
    private ScheduledFuture<?> heartbeat;

    public ControlHandler(
            OrbienClientConfig config,
            DataConnFactory dataFactory,
            Consumer<String> onLoginOk,
            Consumer<Throwable> onFailure) {
        this.config = config;
        this.dataFactory = dataFactory;
        this.onLoginOk = onLoginOk;
        this.onFailure = onFailure;
    }

    public String sessionId() {
        return sessionId;
    }

    @Override
    protected void channelRead0(ChannelHandlerContext ctx, WireMessage msg) {
        switch (msg.type()) {
            case MsgType.LOGIN_RESP -> handleLoginResp(ctx, msg.body());
            case MsgType.REQ_DATA_CONN -> dataFactory.openDataConn(sessionId);
            case MsgType.NEW_TUNNEL_RESP -> handleNewTunnelResp(msg.body());
            case MsgType.PONG -> log.trace("received Pong");
            case MsgType.KICK_OUT -> {
                KickOut k = msg.body();
                log.warn("disconnected by server: {}", k.reason);
                ctx.close();
            }
            default -> log.warn("unsupported control message type={}", (char) msg.type());
        }
    }

    private void handleLoginResp(ChannelHandlerContext ctx, LoginResp resp) {
        if (resp.error != null && !resp.error.isEmpty()) {
            onFailure.accept(new IllegalStateException("login failed: " + resp.error));
            ctx.close();
            return;
        }
        this.sessionId = resp.sessionId == null ? "" : resp.sessionId;
        log.debug("login succeeded, sessionId={}", sessionId);
        registerTunnels(ctx);
        startHeartbeat(ctx);
        onLoginOk.accept(sessionId);
    }

    private void registerTunnels(ChannelHandlerContext ctx) {
        if (!tunnelsRegistered.compareAndSet(false, true)) {
            return;
        }
        for (OrbienClientConfig.TunnelConfig p : config.getTunnels()) {
            String type = p.getProtocol() == null ? "" : p.getProtocol().toLowerCase();
            if (!"tcp".equals(type) && !"http".equals(type)) {
                log.warn("unsupported tunnel protocol={} name={}", type, p.getName());
                continue;
            }
            NewTunnel np = new NewTunnel();
            np.tunnelName = p.getName();
            np.protocol = type;
            np.localIp = p.getLocalIp() == null ? "" : p.getLocalIp();
            np.localPort = p.getLocalPort();
            if ("tcp".equals(type)) {
                np.remotePort = p.getRemotePort();
            } else {
                np.remotePort = 0;
                np.domains = new java.util.ArrayList<>(p.getDomains());
            }
            ctx.writeAndFlush(new WireMessage(MsgType.NEW_TUNNEL, np));
            log.debug(
                    "NewTunnel requested name={} type={} local={}:{} remotePort={} domains={}",
                    p.getName(),
                    type,
                    p.getLocalIp(),
                    p.getLocalPort(),
                    np.remotePort,
                    np.domains);
        }
    }

    private void handleNewTunnelResp(NewTunnelResp resp) {
        if (resp.error != null && !resp.error.isEmpty()) {
            log.error("tunnel registration failed name={} error={}", resp.tunnelName, resp.error);
            return;
        }
        OrbienClientConfig.TunnelConfig tunnel = findTunnel(resp.tunnelName);
        String local =
                tunnel == null ? "?" : tunnel.getLocalIp() + ":" + tunnel.getLocalPort();
        String remote = formatRemoteAddr(resp.remoteAddr, tunnel);
        log.info(
                """
                        
                        ============================================================
                         Tunnel ready [{}]: {} -> {}
                        ============================================================
                        """,
                resp.tunnelName,
                local,
                remote);
    }

    private OrbienClientConfig.TunnelConfig findTunnel(String name) {
        if (name == null) {
            return null;
        }
        for (OrbienClientConfig.TunnelConfig p : config.getTunnels()) {
            if (name.equals(p.getName())) {
                return p;
            }
        }
        return null;
    }

    private String formatRemoteAddr(String remoteAddr, OrbienClientConfig.TunnelConfig tunnel) {
        String remote = remoteAddr == null ? "" : remoteAddr.trim();
        if (remote.startsWith(":")) {
            remote = config.getServerAddr() + remote;
        } else if (remote.isEmpty()) {
            if (tunnel != null && tunnel.getRemotePort() > 0) {
                remote = config.getServerAddr() + ":" + tunnel.getRemotePort();
            } else {
                return "?";
            }
        }
        if (remote.startsWith("http://") || remote.startsWith("https://")) {
            return remote;
        }
        String protocol =
                tunnel != null && tunnel.getProtocol() != null
                        ? tunnel.getProtocol().toLowerCase()
                        : "";
        if ("http".equals(protocol)) {
            return "http://" + remote;
        }
        if ("https".equals(protocol)) {
            return "https://" + remote;
        }
        return remote;
    }

    private void startHeartbeat(ChannelHandlerContext ctx) {
        int interval = Math.max(config.getHeartbeatIntervalSecs(), 0);
        if (interval <= 0) {
            return;
        }
        heartbeat = ctx.executor().scheduleAtFixedRate(() -> {
                    if (!ctx.channel().isActive()) {
                        return;
                    }
                    long ts = System.currentTimeMillis() / 1000;
                    Ping ping = new Ping();
                    ping.timestamp = ts;
                    ping.authDigest = AuthKeys.computeAuthDigest(config.getToken(), ts);
                    ctx.writeAndFlush(new WireMessage(MsgType.PING, ping));
                },
                interval,
                interval,
                TimeUnit.SECONDS);
    }

    @Override
    public void channelInactive(ChannelHandlerContext ctx) {
        if (heartbeat != null) {
            heartbeat.cancel(false);
        }
        log.info("control connection closed");
    }

    @Override
    public void exceptionCaught(ChannelHandlerContext ctx, Throwable cause) {
        log.error("control connection error", cause);
        onFailure.accept(cause);
        ctx.close();
    }
}
