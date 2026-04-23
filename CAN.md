# DFR CAN Standard

## CAN ID Structure (29-bit extended ID)

| Bits    | Field    | Width   | Description                     |
|---------|----------|---------|---------------------------------|
| [28:26] | Priority | 3 bits  | Message priority (0–7)          |
| [25:21] | Target   | 5 bits  | Destination device ID           |
| [20:5]  | Command  | 16 bits | Command/message type            |
| [4:0]   | Source   | 5 bits  | Originating device ID           |

---

## Device IDs (5 bits, range 0x00–0x1F)

| Name             | Hex    | Binary  | Description                              |
|------------------|--------|---------|------------------------------------------|
| `NODE_UNKNOWN`   | `0x00` | `00000` | Unknown / uninitialized device           |
| `NODE_ALL`       | `0x01` | `00001` | Broadcast — targets all nodes            |
| `NODE_FL`        | `0x02` | `00010` | Front-left wheel node                    |
| `NODE_FR`        | `0x03` | `00011` | Front-right wheel node                   |
| `NODE_RL`        | `0x04` | `00100` | Rear-left wheel node                     |
| `NODE_RR`        | `0x05` | `00101` | Rear-right wheel node                    |
| `NODE_NUCLEO_1`  | `0x06` | `00110` | Nucleo board 1                           |
| `NODE_NUCLEO_2`  | `0x07` | `00111` | Nucleo board 2                           |
| `NODE_BMS`       | `0x1C` | `11111` | Battery Management System                |
| `NODE_DASH`      | `0x1D` | `11101` | Dashboard                                |
| `NODE_RASPI`     | `0x1E` | `11110` | Raspberry Pi (main controller)           |

---

## Commands (16 bits, range 0x0000–0xFFFF)

### DAQ Application Commands

| Name               | Hex      | Binary               | Description                                |
|--------------------|----------|----------------------|--------------------------------------------|
| `CMD_PING`         | `0x0001` | `0000000000000001`   | Ping a device to check presence/mode      |
| `CMD_PONG`         | `0x0060` | `0000000001100000`   | Ping response (application mode)          |
| `CMD_REQ_DATA`     | `0x0050` | `0000000001010000`   | Request data from a device                |
| `CMD_SENDING_DATA` | `0x0051` | `0000000001010001`   | Response carrying requested data          |
| `CMD_RESET_NODE`   | `0x0099` | `0000000010011001`   | Soft reset a node                         |
| `CMD_SET_LED`      | `0x0100` | `0000000100000000`   | Set LED state on a device                 |
| `CMD_SET_FREQ`     | `0x0101` | `0000000100000001`   | Set data broadcast frequency on a device  |

### BMS Application Commands
| Name               | Hex      | Binary               | Description                                |
|--------------------|----------|----------------------|--------------------------------------------|
| `CMD_PING`         | `0x0001` | `0000000000000001`   | Ping a device to check presence/mode       |

### Bootloader Commands

| Name               | Hex      | Binary               | Description                                      |
|--------------------|----------|----------------------|--------------------------------------------------|
| `BL_CMD_PING`      | `0x0040` | `0000000001000000`   | Ping a device in bootloader mode               |
| `BL_CMD_ERASE`     | `0x0045` | `0000000001000101`   | Erase flash before writing new firmware        |
| `BL_CMD_ERASE_OK`  | `0x0046` | `0000000001000110`   | Acknowledge successful flash erase             |
| `BL_CMD_WRITE`     | `0x0047` | `0000000001000111`   | Write firmware chunk to device                 |
| `BL_CMD_WRITE_OK`  | `0x0048` | `0000000001001000`   | Acknowledge successful firmware chunk write    |
| `BL_CMD_ADDR_SIZE` | `0x004A` | `0000000001001010`   | Send target address and data size to device    |
| `BL_CMD_FW_QUERY`  | `0x004B` | `0000000001001011`   | Query current firmware version on device       |
| `BL_CMD_FW_RESP`   | `0x004C` | `0000000001001100`   | Response carrying firmware version info        |
| `BL_CMD_REBOOT`    | `0x004D` | `0000000001001101`   | Reboot a device (stay in bootloader)           |
| `BL_CMD_JUMP`      | `0xAAAA` | `1010101010101010`   | Jump from bootloader to application firmware   |
