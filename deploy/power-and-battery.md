# Power and Battery — Galaxy Tab A11

Notes on the always-plugged setup: power adequacy and long-term battery health.

---

## Battery protection

Samsung One UI includes a **Battery protection** mode designed for devices that stay plugged in permanently.

**Setting:** Settings → Battery → Battery protection → **Maximum**

| Mode | Behaviour |
|---|---|
| Off | Charges to 100% and stays there |
| Basic | Charges to 100%, stops, resumes at 95% |
| **Maximum** *(selected)* | Charges to a user-set ceiling (85% recommended) |

Maximum at 85% is the right choice for an always-on device. High state of charge is the primary driver of lithium-ion degradation. Keeping the ceiling at 85% meaningfully extends the battery's lifespan with no practical downside — the tablet remains plugged in and 85% is more than enough charge for any time it is unplugged.

---

## PC USB-C port power output

The PC's USB-C port power output has not been formally measured (no inline USB power meter available). HWiNFO64 was checked but the motherboard does not expose USB port current as a sensor.

**How to monitor in practice:** watch the battery level indicator in daily use. If the level holds steady at or near 85% (the battery protection ceiling), the port is delivering enough power. If the level trends downward during normal use, the port is not keeping up and a powered USB hub is the fix.

**If a USB-C power meter becomes available**, measure voltage and current inline between the PC port and the tablet cable. Watts = V × A. A USB-C port delivering 5V/1.5A (7.5W) or better should be sufficient for the tablet at normal screen brightness.

---

## Summary

| Item | Status |
|---|---|
| Battery protection | Maximum, 85% ceiling — configured |
| PC port power output | Not measured — monitor battery level in daily use |
