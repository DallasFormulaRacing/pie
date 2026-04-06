import { useDaqSocket } from "@/hooks/backendSocket";
import { CommonLayout } from "@/components/CommonLayout";
import { NodeStatus } from "@/components/NodeStatus";
import { LiveTestGraph } from "@/components/Graph";
import { LiveCellVoltage } from "@/components/CellVoltage";
import { Wifi, WifiOff } from "lucide-react";
import { cn } from "@/lib/utils";
import { useState, useEffect } from "react";
import type { SensorReading } from "@/types/backend";

const WS_URL = `ws://localhost:9002`;

export function DaqMonitor() {

  const { devices, data, connected, sendCommand } = useDaqSocket(WS_URL);
  
  interface DaqState {
    cells1: SensorReading[],
    cells2: SensorReading[],
    cells3: SensorReading[],
    cells4: SensorReading[],
    cells5: SensorReading[],
    cells6: SensorReading[],
    temps1: SensorReading[],
    temps2: SensorReading[]
  };

  const [cellData, setCellData] = useState<DaqState>({
    cells1: [],
    cells2: [],
    cells3: [],
    cells4: [],
    cells5: [],
    cells6: [],
    temps1: [],
    temps2: []
  });

  useEffect(() => {
      if (!data) return;

      setCellData((prev) => {
        console.log("Current cells1 length:", prev.cells1.length);
        switch (data.cmd) {
          case "First 24 Cells":
            console.log("cells1: ", data.sensors.length);
            return { ...prev, cells1: data.sensors };
          case "Second 24 Cells":
            return { ...prev, cells2: data.sensors };
          case "Third 24 Cells":
            return { ...prev, cells3: data.sensors};
          case "Fourth 24 Cells":
            return { ...prev, cells4: data.sensors};
          case "Fifth 24 Cells":
            return { ...prev, cells5: data.sensors};
          case "Sixth 24 Cells":
            return { ...prev, cells6: data.sensors};
          case "First 60 Temps":
            return { ...prev, temps1: data.sensors };
          case "Last 60 Temps":
            return { ...prev, temps2: data.sensors };
          default:
            return prev;
        }
      });
    }, [data]);

  return (
    <CommonLayout>
      <div className="grid grid-cols-1 xl:grid-cols-5 gap-8">
        {/* Left: Telemetry Graphs */}
        <div className="xl:col-span-4 grid grid-cols-1 lg:grid-cols-2 gap-4">
          <LiveTestGraph
            title={"First 24 Cells Sensor Data"}
            description={`Real-time ${data?.source || "DAQ"} feed`}
            sensorData={cellData.cells1}
          />
          <LiveTestGraph
            title={"Second 24 Cells Sensor Data"}
            description={`Real-time ${data?.source || "DAQ"} feed`}
            sensorData={cellData.cells2}
          />
          <LiveTestGraph
            title={"Third 24 Cells Sensor Data"}
            description={`Real-time ${data?.source || "DAQ"} feed`}
            sensorData={cellData.cells3}
          />
          <LiveTestGraph
            title={"Fourth 24 Cells Sensor Data"}
            description={`Real-time ${data?.source || "DAQ"} feed`}
            sensorData={cellData.cells4}
          />
          <LiveTestGraph
            title={"Fifth 24 Cells Sensor Data"}
            description={`Real-time ${data?.source || "DAQ"} feed`}
            sensorData={cellData.cells5}
          />
          <LiveTestGraph
            title={"Sixth 24 Cells Sensor Data"}
            description={`Real-time ${data?.source || "DAQ"} feed`}
            sensorData={cellData.cells6}
          />
          <LiveTestGraph
            title={"First 60 Temps Data"}
            description={`Real-time ${data?.source || "DAQ"} feed`}
            sensorData={cellData.temps1}
          />
          <LiveTestGraph
            title={"Last 60 Temps Data"}
            description={`Real-time ${data?.source || "DAQ"} feed`}
            sensorData={cellData.temps2}
          />
        </div>

        {/* Right: Node Management Sidebar */}
        <div className="xl:col-span-1 flex flex-col gap-4">
          <div className="flex items-center justify-between">
            <h3 className="text-sm font-semibold text-muted-foreground uppercase tracking-wider">
              System Nodes
            </h3>
            <div className="flex items-center gap-1.5">
              {connected ? (
                <Wifi className="h-3.5 w-3.5 text-green-500" />
              ) : (
                <WifiOff className="h-3.5 w-3.5 text-red-500" />
              )}
              <span
                className={cn(
                  "text-[10px] uppercase font-medium",
                  connected ? "text-green-500" : "text-red-500",
                )}
              >
                {connected ? "Live" : "Disconnected"}
              </span>
            </div>
          </div>

          <div className="grid grid-cols-2 xl:grid-cols-1 gap-2">
            {devices.map((device) => (
              <NodeStatus
                key={device.deviceId}
                device={device}
                onPing={(id) =>
                  sendCommand({ cmd: "pingDevice", deviceId: id })
                }
                onReboot={(id) =>
                  sendCommand({ cmd: "rebootDevice", deviceId: id })
                }
              />
            ))}
          </div>
        </div>
      </div>
    </CommonLayout>
  );
}

export default DaqMonitor;
