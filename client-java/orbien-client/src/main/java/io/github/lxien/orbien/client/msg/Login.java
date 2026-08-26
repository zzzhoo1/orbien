package io.github.lxien.orbien.client.msg;

import com.fasterxml.jackson.databind.PropertyNamingStrategies;
import com.fasterxml.jackson.databind.annotation.JsonNaming;

@JsonNaming(PropertyNamingStrategies.SnakeCaseStrategy.class)
public class Login {
    public String version = "";
    public String hostname = "";
    public String os = "";
    public String arch = "";
    public String user = "";
    public String authDigest = "";
    public long timestamp;
    public String sessionId = "";
    public int poolCount;
}
