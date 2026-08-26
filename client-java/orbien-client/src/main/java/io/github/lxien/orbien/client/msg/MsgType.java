package io.github.lxien.orbien.client.msg;

public final class MsgType {
    public static final byte LOGIN = 'A';
    public static final byte LOGIN_RESP = 'a';
    public static final byte NEW_TUNNEL = 'T';
    public static final byte NEW_TUNNEL_RESP = 't';
    public static final byte NEW_DATA_CONN = 'W';
    public static final byte REQ_DATA_CONN = 'Q';
    public static final byte START_DATA_CONN = 'S';
    public static final byte PING = 'G';
    public static final byte PONG = 'g';
    public static final byte KICK_OUT = 'E';

    private MsgType() {}
}
