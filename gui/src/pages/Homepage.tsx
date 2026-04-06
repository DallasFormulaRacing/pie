// src/pages/DaqHome.tsx
import { CommonLayout } from "@/components/CommonLayout";
import { InfoBox } from "@/components/InfoBox";
import { TextBoxes } from "@/components/TextBoxes";
import { useEffect, useState } from "react";
import type { SensorReading } from "@/types/backend";
import { useDaqSocket } from "@/hooks/backendSocket";

const WS_URL = `ws://localhost:9002`;

export function Homepage() {
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
    // need confirmation for each of these
    <CommonLayout>
      <div className="grid grid-cols-1 lg:grid-cols-4 gap-4">
        <InfoBox 
          title={"Total Voltage"}
          inputData={0}
        />
        <InfoBox 
          title={"Average Temp"}
          inputData={0}
        />
        <InfoBox 
          title={"State of Charge"}
          inputData={0}
        />
        <InfoBox 
          title={"Power Status"}
          inputData={0}
        />
        <InfoBox 
          title={"Cell Balancing"}
          inputData={0}
        />
        <InfoBox 
          title={"Pack Current"}
          inputData={0}
        />
        <InfoBox 
          title={"Pack Power"}
          inputData={0}
        />
        <InfoBox 
          title={"Voltage Range"}
          inputData={0}
        />
      </div>
      <div className="flex justify-center">
        <TextBoxes 
          sendCommand={sendCommand}
        />
      </div>
    </CommonLayout>
  );
}

export default Homepage;