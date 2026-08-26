---
sidebar_position: 90
sidebar_label: Performance
---

# Performance

| Env | Details |
|-----|---------|
| Platform | macOS 26.2 · Darwin 25.2.0 · arm64 |
| Hardware | Apple M2 (8 cores) · 16 GB |
| Details | [benchmarks](https://github.com/orbien-org/benchmarks) |

To rule out various sources of interference, these benchmarks were run on local loopback. Compared with `frp`, Orbien's clearest advantage is lower and steadier memory usage under high concurrency.

![mem-graph.png](_img/bench/mem-graph.png)

![tcp-bitrate.png](_img/bench/tcp-bitrate.png)

![udp-bitrate.png](_img/bench/udp-bitrate.png)

![http-throughput.png](_img/bench/http-throughput.png)
