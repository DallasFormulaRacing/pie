// src/hooks/useDaqSocket.ts
import { useState, useEffect, useCallback, useRef } from 'react';
import type { UiUpdate, UiCommand } from '../types/daq';

export function useDaqSocket(url: string) {
  const [nodes, setNodes] = useState<Record<string, boolean>>({});
  const socket = useRef<WebSocket | null>(null);

  useEffect(() => {
    socket.current = new WebSocket(url);

    socket.current.onmessage = (event) => {
      const update: UiUpdate = JSON.parse(event.data);
      
      switch (update.type) {
        case "NODE_STATUS":
          setNodes(prev => ({ 
            ...prev, 
            [update.data.node]: update.data.is_online 
          }));
          break;
        case "TELEMETRY":
          break;
      }
    };

    return () => socket.current?.close();
  }, [url]);

  const sendCommand = useCallback((cmd: UiCommand) => {
    if (socket.current?.readyState === WebSocket.OPEN) {
      socket.current.send(JSON.stringify(cmd));
    }
  }, []);

  return { nodes, sendCommand };
}