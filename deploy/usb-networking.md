# USB Networking — Sentinel

Documents how the USB-C link between the Galaxy Tab A11 and the Windows PC is established and verified.

## How it works

The tablet uses Android USB tethering (RNDIS), which makes Windows see it as a standard Ethernet adapter. The tablet acts as a DHCP server and assigns the PC an IP on the USB subnet. The same cable carries both power and data.

## Setup steps

1. Connect the USB-C cable between the tablet and the PC.
2. On the tablet: **Settings → Connections → Mobile Hotspot and Tethering → USB Tethering** → toggle on.
   > The USB Tethering toggle only appears when a cable is connected.
3. Windows will detect a new network adapter automatically — no driver install needed.

## Addresses

| Device | Address | Notes |
|---|---|---|
| Tablet (panel) | `10.186.188.235` | Fixed — Android always assigns itself this address on the USB tethering interface |
| PC (agent) | `10.186.188.x` | Dynamic — assigned by the tablet's DHCP server; changes on reconnect |

The agent dials out to the tablet, so only the tablet's address matters. The PC's address is irrelevant to Sentinel — no static IP or DHCP reservation is needed on the PC side.

## Verification

**Ping (ICMP):**
```powershell
ping 10.186.188.235
```
Expected: 4/4 replies, sub-millisecond RTT.

**TCP (once panel is deployed):**
```powershell
Test-NetConnection -ComputerName 10.186.188.235 -Port 9000
```
Expected after deploy: `TcpTestSucceeded : True` and `InterfaceAlias : Ethernet 2`.

Before deploy, `TcpTestSucceeded` will be `False` with `InterfaceAlias : Ethernet 2` — this confirms TCP routing works over the USB link even though port 9000 is not yet open.

## Windows adapter

Windows lists the tethering adapter as **Ethernet 2** (name may vary). Confirm traffic routes over this adapter and not Wi-Fi by checking `InterfaceAlias` in the `Test-NetConnection` output.
