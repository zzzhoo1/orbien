package io.github.lxien.orbien.client.msg;

import com.fasterxml.jackson.databind.PropertyNamingStrategies;
import com.fasterxml.jackson.databind.annotation.JsonNaming;

@JsonNaming(PropertyNamingStrategies.SnakeCaseStrategy.class)
public class Ping {
    public String authDigest = "";
    public long timestamp;
}
