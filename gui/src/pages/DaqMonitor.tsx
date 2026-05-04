import { useMemo } from "react";
import { useDaqSocket } from "@/hooks/backendSocket";
import { CommonLayout } from "@/components/CommonLayout";
import { NodeStatus } from "@/components/NodeStatus";
import { LiveTestGraph } from "@/components/Graph";
import { Wifi, WifiOff } from "lucide-react";
import { cn } from "@/lib/utils";

const WS_URL = `ws://pi.local:9002`;
export function DaqMonitor() {
  const { devices, connected, imuSeries, temperatureSeries } =
    useDaqSocket(WS_URL);
  const accelSeries = useMemo(
    () => [
      {
        label: "X",
        values: imuSeries.accelX,
        stroke: "oklch(0.646 0.222 41.116)",
      },
      {
        label: "Y",
        values: imuSeries.accelY,
        stroke: "oklch(0.72 0.18 145)",
      },
      {
        label: "Z",
        values: imuSeries.accelZ,
        stroke: "oklch(0.68 0.2 260)",
      },
    ],
    [imuSeries.accelX, imuSeries.accelY, imuSeries.accelZ],
  );
  const gyroSeries = useMemo(
    () => [
      {
        label: "X",
        values: imuSeries.gyroX,
        stroke: "oklch(0.646 0.222 41.116)",
      },
      {
        label: "Y",
        values: imuSeries.gyroY,
        stroke: "oklch(0.72 0.18 145)",
      },
      {
        label: "Z",
        values: imuSeries.gyroZ,
        stroke: "oklch(0.68 0.2 260)",
      },
    ],
    [imuSeries.gyroX, imuSeries.gyroY, imuSeries.gyroZ],
  );
  const temperatureGraphSeries = useMemo(
    () => [
      {
        label: "Tire Avg",
        values: temperatureSeries.tireAverage,
        stroke: "oklch(0.646 0.222 41.116)",
      },
      {
        label: "Brake Avg",
        values: temperatureSeries.brakeAverage,
        stroke: "oklch(0.72 0.18 145)",
      },
    ],
    [temperatureSeries.tireAverage, temperatureSeries.brakeAverage],
  );

  return (
    <CommonLayout>
      <div className="grid grid-cols-1 xl:grid-cols-5 gap-8">
        <div className="xl:col-span-4 grid grid-cols-1 lg:grid-cols-2 gap-4">
          <LiveTestGraph
            title="IMU Accel"
            description="Acceleration (g)"
            time={imuSeries.time}
            series={accelSeries}
          />
          <LiveTestGraph
            title="IMU Gyro"
            description="Angular rate (dps)"
            time={imuSeries.time}
            series={gyroSeries}
          />
          <LiveTestGraph
            title="Temperature"
            description="Average tire and brake temperature (C)"
            time={temperatureSeries.time}
            series={temperatureGraphSeries}
          />
        </div>

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
              <NodeStatus key={device.nodeId} device={device} />
            ))}
          </div>
        </div>
      </div>
    </CommonLayout>
  );
}

export default DaqMonitor;
