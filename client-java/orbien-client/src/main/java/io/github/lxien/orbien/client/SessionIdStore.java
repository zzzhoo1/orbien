package io.github.lxien.orbien.client;

import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

final class SessionIdStore {
    private static final Logger log = LoggerFactory.getLogger(SessionIdStore.class);

    static final String DEFAULT_FILE = ".orbien.session_id";
    private SessionIdStore() {}

    static Path defaultPath() {
        return Path.of(DEFAULT_FILE);
    }

    static String load(Path path) {
        if (path == null || !Files.isRegularFile(path)) {
            return "";
        }
        try {
            String s = Files.readString(path, StandardCharsets.UTF_8).trim();
            if (s.isEmpty() || s.length() > 64) {
                return "";
            }
            for (int i = 0; i < s.length(); i++) {
                char c = s.charAt(i);
                if (!(c >= '0' && c <= '9'
                        || c >= 'a' && c <= 'f'
                        || c >= 'A' && c <= 'F'
                        || c == '-')) {
                    return "";
                }
            }
            return s;
        } catch (IOException e) {
            log.warn("failed to load sessionId from {}: {}", path, e.toString());
            return "";
        }
    }

    static void save(Path path, String sessionId) {
        if (path == null || sessionId == null || sessionId.isBlank()) {
            return;
        }
        try {
            Path parent = path.getParent();
            if (parent != null) {
                Files.createDirectories(parent);
            }
            Files.writeString(path, sessionId, StandardCharsets.UTF_8);
        } catch (IOException e) {
            log.warn("failed to persist sessionId to {}: {}", path, e.toString());
        }
    }
}
