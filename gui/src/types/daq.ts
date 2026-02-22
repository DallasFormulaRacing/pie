// src/types/daq.ts
export type UiUpdate = 
  | { type: "NODE_STATUS"; data: { node: string; is_online: boolean } }
  | { type: "TELEMETRY"; data: { id: string; data: string } }
  | { type: "STATUS"; data: string };

export type UiCommand = 
  | { command: "SET_POLLING"; payload: boolean }
  | { command: "TOGGLE_LED"; payload: { node: string; state: boolean } }
  | { command: "REBOOT"; payload: { node: string } }
  | { command: "BOOTLOADER"; payload: { node: string } }
  | { command: "REFRESH_NODES" } 
  | { command: "PING_NODE"; payload: { node: string } };