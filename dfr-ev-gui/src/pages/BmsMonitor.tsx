import { CommonLayout } from "@/components/CommonLayout";
import { LiveTestGraph } from "@/components/Graph";

export function BmsMonitor() {
  return (
    <CommonLayout>
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-8">
        <LiveTestGraph title="Motor RPM" description="Main Drive Motor Speed" />
        <LiveTestGraph title="Battery Pack" description="Total Bus Voltage (V)" />
        <LiveTestGraph title="Inverter Temp" description="Coolant Loop B Status" />
        <LiveTestGraph title="Tire Pressure" description="Front Left Sensor" />
      </div>
    </CommonLayout>
  );
}

export default BmsMonitor;