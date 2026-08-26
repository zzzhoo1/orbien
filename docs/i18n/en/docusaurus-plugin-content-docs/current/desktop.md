---
sidebar_position: 5
sidebar_label: Desktop Client
title: Desktop Client
---

# Desktop Client

## Installation issues

### macOS cannot open the app or reports it as damaged

![desktop_mac_1.png](_img/desktop_mac_1.png)

Downloaded `.app` / `.dmg` packages may be quarantined by the system. If the app cannot be opened, or macOS reports that it is damaged, run the following in Terminal:

```shell
xattr -cr "/Applications/Orbien Desktop.app"
open "/Applications/Orbien Desktop.app"
```

If the app is installed elsewhere, replace the path above with the actual `.app` location before running the commands.
