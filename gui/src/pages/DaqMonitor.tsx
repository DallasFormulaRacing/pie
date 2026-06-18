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
  
  const daqSource = data?.source || "DAQ";
  const graphDescription = `Real-time ${daqSource} feed`;

  const [cells1, setCells1] = useState<SensorReading[]>([]);
  const [cells2, setCells2] = useState<SensorReading[]>([]);
  const [cells3, setCells3] = useState<SensorReading[]>([]);
  const [cells4, setCells4] = useState<SensorReading[]>([]);
  const [cells5, setCells5] = useState<SensorReading[]>([]);
  const [cells6, setCells6] = useState<SensorReading[]>([]);
  const [temps1, setTemps1] = useState<SensorReading[]>([]);
  const [temps2, setTemps2] = useState<SensorReading[]>([]);

  useEffect(() => {
      if (!data) return;

      switch (data.cmd) {
        case "First 24 Cells":
          console.log("cells1: ", data.sensors);
          setCells1(data.sensors);
          break;
        case "Second 24 Cells":
          console.log("cells2: ", data.sensors);
          setCells2(data.sensors);
          break;
        case "Third 24 Cells":
          setCells3(data.sensors);
          break;
        case "Fourth 24 Cells":
          setCells4(data.sensors);
          break;
        case "Fifth 24 Cells":
          setCells5(data.sensors);
          break;
        case "Sixth 24 Cells":
          setCells6(data.sensors);
          break;
        case "First 60 Temps":
          setTemps1(data.sensors);
          break;
        case "Last 60 Temps":
          setTemps2(data.sensors);
          break;
        default:
          break;
      }
    }, [data]);

  return (
    <CommonLayout>
      <div className="grid grid-cols-1 xl:grid-cols-5 gap-8">
        {/* Left: Telemetry Graphs */}
        <div className="xl:col-span-4 grid grid-cols-1 lg:grid-cols-2 gap-4">
          <LiveTestGraph
            title={"First 24 Cells Sensor Data"}
            description={graphDescription}
            sensorData={cells1}
          />
          <LiveTestGraph
            title={"Second 24 Cells Sensor Data"}
            description={graphDescription}
            sensorData={cells2}
          />
          <LiveTestGraph
            title={"Third 24 Cells Sensor Data"}
            description={graphDescription}
            sensorData={cells3}
          />
          <LiveTestGraph
            title={"Fourth 24 Cells Sensor Data"}
            description={graphDescription}
            sensorData={cells4}
          />
          <LiveTestGraph
            title={"Fifth 24 Cells Sensor Data"}
            description={graphDescription}
            sensorData={cells5}
          />
          <LiveTestGraph
            title={"Sixth 24 Cells Sensor Data"}
            description={graphDescription}
            sensorData={cells6}
          />
          <LiveTestGraph
            title={"First 60 Temps Data"}
            description={graphDescription}
            sensorData={temps1}
          />
          <LiveTestGraph
            title={"Last 60 Temps Data"}
            description={graphDescription}
            sensorData={temps2}
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
