package io.github.lxien.orbien.client;

import io.github.lxien.orbien.client.auth.AuthKeys;
import io.github.lxien.orbien.client.msg.Login;
import io.github.lxien.orbien.client.msg.MsgType;
import io.github.lxien.orbien.client.msg.NewWorkConn;
import io.github.lxien.orbien.client.msg.WireMessage;
import io.github.lxien.orbien.client.netty.ControlHandler;
import io.github.lxien.orbien.client.netty.MsgFrameDecoder;
import io.github.lxien.orbien.client.netty.MsgFrameEncoder;
import io.github.lxien.orbien.client.netty.WorkHandshakeHandler;
import io.netty.bootstrap.Bootstrap;
import io.netty.channel.Channel;
import io.netty.channel.ChannelFuture;
import io.netty.channel.ChannelInitializer;
import io.netty.channel.ChannelOption;
import io.netty.channel.EventLoopGroup;
import io.netty.channel.nio.NioEventLoopGroup;
import io.netty.channel.socket.SocketChannel;
import io.netty.channel.socket.nio.NioSocketChannel;

import java.net.InetAddress;
import java.nio.file.Path;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.atomic.AtomicBoolean;
import java.util.concurrent.atomic.AtomicReference;

import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

public final class OrbienClient implements AutoCloseable {
    private static final Logger log = LoggerFactory.getLogger(OrbienClient.class);
    private static final String VERSION = "0.1.0";

    private final OrbienClientConfig config;
    private final AtomicBoolean started = new AtomicBoolean(false);
    private EventLoopGroup group;
    private Channel controlChannel;
    private final AtomicReference<String> runId = new AtomicReference<>("");

    public OrbienClient(OrbienClientConfig config) {
        this.config = config;
    }

    public OrbienClientConfig config() {
        return config;
    }

    public String runId() {
        return runId.get();
    }

    public void start() {
        if (!started.compareAndSet(false, true)) {
            return;
        }
        if (config.isTcpMux()) {
            started.set(false);
            throw new IllegalStateException("tcpMux is not supported; set tcpMux=false on client and server");
        }

        group = new NioEventLoopGroup();
        CompletableFuture<String> loginFuture = new CompletableFuture<>();
        Path runIdPath = resolveRunIdPath();
        String previousRunId = resolvePreviousRunId(runIdPath);

        try {
            Bootstrap b = new Bootstrap();
            b.group(group)
                    .channel(NioSocketChannel.class)
                    .option(ChannelOption.TCP_NODELAY, true)
                    .option(ChannelOption.CONNECT_TIMEOUT_MILLIS, 10_000)
                    .handler(new ChannelInitializer<SocketChannel>() {
                        @Override
                        protected void initChannel(SocketChannel ch) {
                            ch.pipeline()
                                    .addLast(new MsgFrameDecoder())
                                    .addLast(new MsgFrameEncoder())
                                    .addLast(new ControlHandler(
                                            config,
                                            OrbienClient.this::openWorkConn,
                                            id -> {
                                                runId.set(id);
                                                loginFuture.complete(id);
                                            },
                                            loginFuture::completeExceptionally));
                        }
                    });

            ChannelFuture cf =
                    b.connect(config.getServerAddr(), config.getServerPort()).sync();
            controlChannel = cf.channel();
            sendLogin(controlChannel, previousRunId);

            String id = loginFuture.get(30, TimeUnit.SECONDS);
            config.setRunId(id);
            RunIdStore.save(runIdPath, id);
            log.info("connected to {}:{} runId={}", config.getServerAddr(), config.getServerPort(), id);
        } catch (Exception e) {
            close();
            throw new IllegalStateException("failed to start Orbien client: " + e.getMessage(), e);
        }
    }

    private Path resolveRunIdPath() {
        String configured = config.getRunIdFile();
        if (configured != null && !configured.isBlank()) {
            return Path.of(configured);
        }
        return RunIdStore.defaultPath();
    }

    private String resolvePreviousRunId(Path runIdPath) {
        if (config.getRunId() != null && !config.getRunId().isBlank()) {
            return config.getRunId().trim();
        }
        String loaded = RunIdStore.load(runIdPath);
        if (!loaded.isEmpty()) {
            log.info("restored runId={} from {}", loaded, runIdPath);
        }
        return loaded;
    }

    private void sendLogin(Channel ch, String previousRunId) {
        long ts = System.currentTimeMillis() / 1000;
        Login login = new Login();
        login.version = VERSION;
        login.hostname = localHostname();
        login.os = System.getProperty("os.name", "");
        login.arch = System.getProperty("os.arch", "");
        login.user = config.getUser();
        login.timestamp = ts;
        login.privilegeKey = AuthKeys.getAuthKey(config.getToken(), ts);
        login.runId = previousRunId == null ? "" : previousRunId;
        login.poolCount = Math.max(config.getPoolCount(), 1);
        ch.writeAndFlush(new WireMessage(MsgType.LOGIN, login));
        log.debug(
                "login sent hostname={} user={} poolCount={} runId={}",
                login.hostname,
                login.user,
                login.poolCount,
                login.runId.isEmpty() ? "<new>" : login.runId);
    }

    private void openWorkConn(String currentRunId) {
        if (group == null || group.isShuttingDown()) {
            return;
        }
        String rid = currentRunId == null || currentRunId.isEmpty() ? runId.get() : currentRunId;
        Bootstrap b = new Bootstrap();
        b.group(group)
                .channel(NioSocketChannel.class)
                .option(ChannelOption.TCP_NODELAY, true)
                .handler(new ChannelInitializer<SocketChannel>() {
                    @Override
                    protected void initChannel(SocketChannel ch) {
                        ch.pipeline()
                                .addLast(new MsgFrameEncoder())
                                .addLast(new WorkHandshakeHandler(config, group));
                    }
                });

        b.connect(config.getServerAddr(), config.getServerPort())
                .addListener(f -> {
                    if (!f.isSuccess()) {
                        log.error("failed to open work connection", f.cause());
                        return;
                    }
                    Channel work = ((ChannelFuture) f).channel();
                    long ts = System.currentTimeMillis() / 1000;
                    NewWorkConn nw = new NewWorkConn();
                    nw.runId = rid;
                    nw.timestamp = ts;
                    nw.privilegeKey = AuthKeys.getAuthKey(config.getToken(), ts);
                    work.writeAndFlush(new WireMessage(MsgType.NEW_WORK_CONN, nw));
                    log.debug("NewWorkConn sent, runId={}", rid);
                });
    }

    private static String localHostname() {
        try {
            String h = InetAddress.getLocalHost().getHostName();
            if (h != null && !h.isBlank()) {
                return h;
            }
        } catch (Exception ignored) {
        }
        return "unknown";
    }

    @Override
    public void close() {
        started.set(false);
        if (controlChannel != null) {
            controlChannel.close();
            controlChannel = null;
        }
        if (group != null) {
            group.shutdownGracefully(0, 2, TimeUnit.SECONDS);
            group = null;
        }
    }
}
