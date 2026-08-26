package io.github.lxien.orbien.client.msg;

import com.fasterxml.jackson.databind.DeserializationFeature;
import com.fasterxml.jackson.databind.ObjectMapper;

import java.io.IOException;

public final class MsgCodec {
    public static final int MAX_BODY = 4 * 1024 * 1024;

    private static final ObjectMapper MAPPER =
            new ObjectMapper().configure(DeserializationFeature.FAIL_ON_UNKNOWN_PROPERTIES, false);

    private MsgCodec() {
    }

    public static ObjectMapper mapper() {
        return MAPPER;
    }

    public static byte[] encodeBody(Object body) {
        try {
            if (body == null) {
                return new byte[]{'{', '}'};
            }
            return MAPPER.writeValueAsBytes(body);
        } catch (IOException e) {
            throw new IllegalArgumentException("json encode failed", e);
        }
    }

    public static WireMessage decode(byte type, byte[] body) throws IOException {
        Object parsed =
                switch (type) {
                    case MsgType.LOGIN -> MAPPER.readValue(body, Login.class);
                    case MsgType.LOGIN_RESP -> MAPPER.readValue(body, LoginResp.class);
                    case MsgType.NEW_TUNNEL -> MAPPER.readValue(body, NewTunnel.class);
                    case MsgType.NEW_TUNNEL_RESP -> MAPPER.readValue(body, NewTunnelResp.class);
                    case MsgType.NEW_DATA_CONN -> MAPPER.readValue(body, NewDataConn.class);
                    case MsgType.REQ_DATA_CONN -> MAPPER.readValue(body, ReqDataConn.class);
                    case MsgType.START_DATA_CONN -> MAPPER.readValue(body, StartDataConn.class);
                    case MsgType.PING -> MAPPER.readValue(body, Ping.class);
                    case MsgType.PONG -> MAPPER.readValue(body, Pong.class);
                    case MsgType.KICK_OUT -> MAPPER.readValue(body, KickOut.class);
                    default -> throw new IOException("unknown message type: " + (type & 0xff));
                };
        return new WireMessage(type, parsed);
    }
}
