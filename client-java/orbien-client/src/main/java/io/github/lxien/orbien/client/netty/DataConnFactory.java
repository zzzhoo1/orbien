package io.github.lxien.orbien.client.netty;

@FunctionalInterface
public interface DataConnFactory {
    void openDataConn(String sessionId);
}
