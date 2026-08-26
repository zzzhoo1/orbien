package io.github.lxien.orbien.client.msg;

import com.fasterxml.jackson.databind.PropertyNamingStrategies;
import com.fasterxml.jackson.databind.annotation.JsonNaming;
import java.util.ArrayList;
import java.util.List;

@JsonNaming(PropertyNamingStrategies.SnakeCaseStrategy.class)
public class NewTunnel {
    public String tunnelName;
    public String protocol;
    public int remotePort;
    public String localIp = "";
    public int localPort;
    public List<String> domains = new ArrayList<>();
}
