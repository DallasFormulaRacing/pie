export type System = "backend" | "bms" | "daq" | "vcu";

export type Device =
  | "bms"
  | "vcu"
  | "raspi"
  | "nodeFL"
  | "nodeFR"
  | "nodeRL"
  | "nodeRR"
  | "nodeDash"
  | "nodeRideHeight"
  | "nodePDMTB"
  | "nodePDMDASH"
  | "nodePDMPCBPanel";

export interface WsEnvelope<T> {
  system: System;
  device: Device;
  data: T;
}

export type BackendEvent = WsEnvelope<BackendEventData>;

export type BackendEventData =
  | { type: "deviceRegistrySnapshot"; devices: DeviceStatus[] }
  | { type: "deviceStatusChanged"; device: DeviceStatus }
  | { type: "daqTelemetry"; telemetry: DaqTelemetry }
  | { type: "bmsTelemetry"; telemetry: BmsTelemetry }
  | { type: "backendError"; message: string };

export type DaqTelemetry =
  | {
      type: "temperature";
      source: Device;
      samples: [
        TemperatureSample,
        TemperatureSample,
        TemperatureSample,
        TemperatureSample,
        TemperatureSample,
        TemperatureSample,
        TemperatureSample,
        TemperatureSample,
        TemperatureSample,
        TemperatureSample,
        TemperatureSample,
        TemperatureSample,
        TemperatureSample,
        TemperatureSample,
        TemperatureSample,
        TemperatureSample,
      ];
    }
  | {
      type: "imu";
      source: Device;
      samples: [ImuSample, ImuSample, ImuSample, ImuSample, ImuSample];
    }
  | { type: "tbd"; source: Device; value: number };

export interface TemperatureSample {
  tire: number;
  brake: number;
}

export interface ImuSample {
  acceleration: Acceleration;
  angularAcceleration: AngularAcceleration;
}

export interface Acceleration {
  x: number;
  y: number;
  z: number;
}

export interface AngularAcceleration {
  rho: number;
  theta: number;
  phi: number;
}

export type BmsTelemetry =
  | { type: "voltages"; source: Device; readings: BmsVoltageReadings }
  | { type: "temperatures"; source: Device; readings: BmsTemperatureReadings }
  | { type: "balancing"; source: Device; activeCell: number; dutyCycle: number }
  | { type: "faults"; source: Device; code: number; severity: number };

export interface BmsVoltageReadings {
  pack: number;
  minCell: number;
  maxCell: number;
  averageCell: number;
}

export interface BmsTemperatureReadings {
  min: number;
  max: number;
  average: number;
}

export interface DeviceStatus {
  nodeId: number;
  name: string;
  system: string;
  online: boolean;
  lastSeenMsAgo: number | null;
  lastError: string | null;
}
