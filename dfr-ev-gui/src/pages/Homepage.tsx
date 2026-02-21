// src/pages/DaqHome.tsx
import { CommonLayout } from "@/components/CommonLayout";
import { LiveTestGraph } from "@/components/Graph";

export function Homepage() {
  return (
    <CommonLayout>
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-8">
        <LiveTestGraph title="Wheel Speed" description="Average Wheel Speed (km/h)" />
        <LiveTestGraph title="Lin Pot" description="Linear Potentiometer Travel (mm)" />
        <LiveTestGraph title="Inverter Temp" description="Inverter Temperature (°C)" />
        <LiveTestGraph title="Tire Temperature" description="Average Tire Temperature (°C)" />
      </div>
    </CommonLayout>
  );
}

export default Homepage;