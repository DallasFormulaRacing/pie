export type BackendMessage =
  | { type: "deviceList"; devices: DeviceStatus[] }
  | { type: "sensorData"; source: string; sensors: SensorReading[] }
  | {
      type: "pingResult";
      deviceId: number;
      online: boolean;
      mode: string | null;
      rttMs: number | null;
    }
  | { type: "error"; message: string };

export interface DeviceStatus {
  deviceId: number;
  name: string;
  online: boolean;
  mode: string; // "application" | "bootloader" | "offline"
}

export interface SensorReading {
  name: string;
  value: number;
  unit: string;
}

export type FrontendCommand =
  | { cmd: "pingDevice"; deviceId: number }
  | { cmd: "rebootDevice"; deviceId: number }
  | { cmd: "getDeviceList" };
