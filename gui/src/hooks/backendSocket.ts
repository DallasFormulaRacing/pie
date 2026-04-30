import { useCallback, useEffect, useRef, useState } from "react";
import type {
  BackendEvent,
  DeviceStatus,
  ImuSample,
  TemperatureSample,
} from "../types/backend";

const RECONNECT_INTERVAL_MS = 2000;
const MAX_IMU_POINTS = 500;
const IMU_SAMPLE_SPACING_SECONDS = 0.0024;
const IMU_UI_FLUSH_INTERVAL_MS = 66;

export interface ImuSeries {
  time: number[];
  accelX: number[];
  accelY: number[];
  accelZ: number[];
  gyroX: number[];
  gyroY: number[];
  gyroZ: number[];
}

export interface TemperatureSeries {
  time: number[];
  tireAverage: number[];
  brakeAverage: number[];
}

const emptyImuSeries: ImuSeries = {
  time: [],
  accelX: [],
  accelY: [],
  accelZ: [],
  gyroX: [],
  gyroY: [],
  gyroZ: [],
};

const emptyTemperatureSeries: TemperatureSeries = {
  time: [],
  tireAverage: [],
  brakeAverage: [],
};

export function useDaqSocket(url: string) {
  const [devices, setDevices] = useState<DeviceStatus[]>([]);
  const [imuSeries, setImuSeries] = useState<ImuSeries>(emptyImuSeries);
  const [temperatureSeries, setTemperatureSeries] = useState<TemperatureSeries>(
    emptyTemperatureSeries,
  );
  const [connected, setConnected] = useState(false);
  const socket = useRef<WebSocket | null>(null);
  const reconnectTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const imuFlushTimer = useRef<ReturnType<typeof setInterval> | null>(null);
  const imuBuffer = useRef<ImuSeries>(emptyImuSeries);
  const temperatureBuffer = useRef<TemperatureSeries>(emptyTemperatureSeries);
  const shouldReconnect = useRef(true);

  const connect = useCallback(() => {
    if (socket.current?.readyState === WebSocket.OPEN) return;

    const ws = new WebSocket(url);
    socket.current = ws;

    ws.onopen = () => {
      setConnected(true);
    };

    ws.onmessage = (event) => {
      let msg: BackendEvent;

      try {
        msg = JSON.parse(event.data) as BackendEvent;
      } catch (error) {
        console.error("Backend websocket sent invalid JSON:", error);
        return;
      }

      const data = msg.data;

      switch (data.type) {
        case "deviceRegistrySnapshot":
          setDevices(data.devices);
          break;
        case "deviceStatusChanged":
          setDevices((current) => upsertDevice(current, data.device));
          break;
        case "daqTelemetry":
          if (data.telemetry.type === "imu") {
            appendImuSamples(imuBuffer.current, data.telemetry.samples);
          } else if (data.telemetry.type === "temperature") {
            appendTemperatureSamples(
              temperatureBuffer.current,
              data.telemetry.samples,
            );
          }
          break;
        case "bmsTelemetry":
          break;
        case "backendError":
          console.error("Backend error:", data.message);
          break;
      }
    };

    ws.onclose = () => {
      setConnected(false);
      socket.current = null;
      if (shouldReconnect.current) {
        reconnectTimer.current = setTimeout(connect, RECONNECT_INTERVAL_MS);
      }
    };

    ws.onerror = () => {
      ws.close();
    };
  }, [url]);

  useEffect(() => {
    shouldReconnect.current = true;
    imuFlushTimer.current = setInterval(() => {
      setImuSeries(cloneImuSeries(imuBuffer.current));
      setTemperatureSeries(cloneTemperatureSeries(temperatureBuffer.current));
    }, IMU_UI_FLUSH_INTERVAL_MS);
    connect();
    return () => {
      shouldReconnect.current = false;
      if (reconnectTimer.current) {
        clearTimeout(reconnectTimer.current);
      }
      if (imuFlushTimer.current) {
        clearInterval(imuFlushTimer.current);
      }
      socket.current?.close();
    };
  }, [connect]);

  return { devices, connected, imuSeries, temperatureSeries };
}

function upsertDevice(devices: DeviceStatus[], changedDevice: DeviceStatus) {
  const existing = devices.some(
    (device) => device.nodeId === changedDevice.nodeId,
  );

  if (!existing) {
    return [...devices, changedDevice].sort((a, b) => a.nodeId - b.nodeId);
  }

  return devices.map((device) =>
    device.nodeId === changedDevice.nodeId ? changedDevice : device,
  );
}

function appendImuSamples(series: ImuSeries, samples: readonly ImuSample[]) {
  const firstTime = Date.now() / 1000;
  samples.forEach((sample, index) => {
    series.time.push(firstTime + index * IMU_SAMPLE_SPACING_SECONDS);
    series.accelX.push(sample.acceleration.x);
    series.accelY.push(sample.acceleration.y);
    series.accelZ.push(sample.acceleration.z);
    series.gyroX.push(sample.angularAcceleration.rho);
    series.gyroY.push(sample.angularAcceleration.theta);
    series.gyroZ.push(sample.angularAcceleration.phi);
  });

  trimImuSeries(series);
}

function trimImuSeries(series: ImuSeries) {
  const removeCount = Math.max(0, series.time.length - MAX_IMU_POINTS);

  if (removeCount === 0) {
    return;
  }

  series.time.splice(0, removeCount);
  series.accelX.splice(0, removeCount);
  series.accelY.splice(0, removeCount);
  series.accelZ.splice(0, removeCount);
  series.gyroX.splice(0, removeCount);
  series.gyroY.splice(0, removeCount);
  series.gyroZ.splice(0, removeCount);
}

function appendTemperatureSamples(
  series: TemperatureSeries,
  samples: readonly TemperatureSample[],
) {
  const tireAverage =
    samples.reduce((sum, sample) => sum + sample.tire, 0) / samples.length;
  const brakeAverage =
    samples.reduce((sum, sample) => sum + sample.brake, 0) / samples.length;

  series.time.push(Date.now() / 1000);
  series.tireAverage.push(tireAverage);
  series.brakeAverage.push(brakeAverage);

  trimTemperatureSeries(series);
}

function trimTemperatureSeries(series: TemperatureSeries) {
  const removeCount = Math.max(0, series.time.length - MAX_IMU_POINTS);

  if (removeCount === 0) {
    return;
  }

  series.time.splice(0, removeCount);
  series.tireAverage.splice(0, removeCount);
  series.brakeAverage.splice(0, removeCount);
}

function cloneImuSeries(series: ImuSeries): ImuSeries {
  return {
    time: [...series.time],
    accelX: [...series.accelX],
    accelY: [...series.accelY],
    accelZ: [...series.accelZ],
    gyroX: [...series.gyroX],
    gyroY: [...series.gyroY],
    gyroZ: [...series.gyroZ],
  };
}

function cloneTemperatureSeries(series: TemperatureSeries): TemperatureSeries {
  return {
    time: [...series.time],
    tireAverage: [...series.tireAverage],
    brakeAverage: [...series.brakeAverage],
  };
}
