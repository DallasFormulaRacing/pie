import { useDaqSocket } from "@/hooks/backendSocket";
import { CommonLayout } from "@/components/CommonLayout";
import { NodeStatus } from "@/components/NodeStatus";
import { LiveTestGraph } from "@/components/Graph";
import { Wifi, WifiOff } from "lucide-react";
import { cn } from "@/lib/utils";

const WS_URL = `ws://0.0.0.0:9002`;

export function DaqMonitor() {
  const { devices, connected, sendCommand } = useDaqSocket(WS_URL);

  return (
    <CommonLayout>
      <div className="grid grid-cols-1 xl:grid-cols-5 gap-8">
        {/* Left: Telemetry Graphs */}
        <div className="xl:col-span-4 grid grid-cols-1 lg:grid-cols-2 gap-4">
          <LiveTestGraph
            title="Wheel Speed"
            description="Average Wheel Speed (km/h)"
          />
          <LiveTestGraph
            title="Lin Pot"
            description="Linear Potentiometer Travel (mm)"
          />
          <LiveTestGraph
            title="Inverter Temp"
            description="Inverter Temperature (°C)"
          />
          <LiveTestGraph
            title="Tire Temperature"
            description="Average Tire Temperature (°C)"
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
