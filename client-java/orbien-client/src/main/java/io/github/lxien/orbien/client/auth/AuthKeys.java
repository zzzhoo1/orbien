package io.github.lxien.orbien.client.auth;

import java.nio.charset.StandardCharsets;
import java.security.InvalidKeyException;
import java.security.NoSuchAlgorithmException;
import java.util.HexFormat;
import javax.crypto.Mac;
import javax.crypto.spec.SecretKeySpec;

public final class AuthKeys {

    private static final String HMAC_SHA256 = "HmacSHA256";

    private AuthKeys() {}

    public static String computeAuthDigest(String token, long timestamp) {
        String t = token == null ? "" : token;
        try {
            Mac mac = Mac.getInstance(HMAC_SHA256);
            mac.init(new SecretKeySpec(t.getBytes(StandardCharsets.UTF_8), HMAC_SHA256));
            mac.update(Long.toString(timestamp).getBytes(StandardCharsets.UTF_8));
            return HexFormat.of().formatHex(mac.doFinal());
        } catch (NoSuchAlgorithmException | InvalidKeyException e) {
            throw new IllegalStateException("HmacSHA256 not available", e);
        }
    }
}
