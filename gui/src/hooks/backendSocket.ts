import { useState, useEffect, useCallback, useRef } from "react";
import type {
  BackendMessage,
  SensorReading,
  DeviceStatus,
  FrontendCommand,
} from "../types/backend";

const RECONNECT_INTERVAL_MS = 2000;

export function useDaqSocket(url: string) {
  const [devices, setDevices] = useState<DeviceStatus[]>([]);
  const [data, setData] = useState<{ source: string; cmd: string; sensors: SensorReading[] } | undefined>();
  const [connected, setConnected] = useState(false);
  const socket = useRef<WebSocket | null>(null);
  const reconnectTimer = useRef<ReturnType<typeof setTimeout> | null>(null);

  const connect = useCallback(() => {
    if (socket.current?.readyState === WebSocket.OPEN) return;

    const ws = new WebSocket(url);
    socket.current = ws;

    ws.onopen = () => {
      setConnected(true);
      ws.send(JSON.stringify({ cmd: "getDeviceList" }));
    };

    ws.onmessage = (event) => {
      const msg: BackendMessage = JSON.parse(event.data);

      switch (msg.type) {
        case "deviceList":
          setDevices(msg.devices);
          console.log("Received deviceList");
          break;
        case "sensorData":
          console.log(msg.sensors)
          setData({ source: msg.source, cmd: msg.cmd, sensors: msg.sensors });
          break;
        case "pingResult":
          break;
        case "error":
          console.error("Backend error:", msg.message);
          break;
      }
    };

    ws.onclose = () => {
      setConnected(false);
      socket.current = null;
      reconnectTimer.current = setTimeout(connect, RECONNECT_INTERVAL_MS);
    };

    ws.onerror = () => {
      ws.close();
    };
  }, [url]);

  useEffect(() => {
    connect();
    return () => {
      reconnectTimer.current && clearTimeout(reconnectTimer.current);
      socket.current?.close();
    };
  }, [connect]);

  const sendCommand = useCallback((cmd: FrontendCommand) => {
    if (socket.current?.readyState === WebSocket.OPEN) {
      socket.current.send(JSON.stringify(cmd));
    }
  }, []);

  return { devices, data, connected, sendCommand };
}
