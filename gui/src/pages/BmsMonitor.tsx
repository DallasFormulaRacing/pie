import { LiveCellVoltage } from "@/components/CellVoltage";
import { CommonLayout } from "@/components/CommonLayout";
import { useDaqSocket } from "@/hooks/backendSocket";
import { Accordion, AccordionContent, AccordionItem, AccordionTrigger } from "@/components/ui/accordion";
import { useState, useEffect } from "react";
import type { SensorReading } from "@/types/backend";

const WS_URL = `ws://localhost:9002`;

// this can be made nicer, kinda looks like it's 
// lagging bc it waits for data before creating each accordion tab

export function BmsMonitor() {
  const { data } = useDaqSocket(WS_URL);
  const [sensorGroups, setSensorGroups] = useState<Record<string, SensorReading[]>>({});

  const cmdOrder = [
    "First 24 Cells",
    "Second 24 Cells",
    "Third 24 Cells",
    "Fourth 24 Cells",
    "Fifth 24 Cells",
    "Sixth 24 Cells",
    "First 60 Temps",
    "Last 60 Temps",
    "Pack Metadata",
    "IMD Data"
  ];

  useEffect(() => {
    if (data?.cmd && data.sensors) {
      console.log("Received data:", data);
      setSensorGroups(prev => ({
        ...prev,
        [data.cmd]: data.sensors
      }));
    }
  }, [data]);

  const sortedGroups = Object.entries(sensorGroups).sort(([a], [b]) => {
    const indexA = cmdOrder.indexOf(a);
    const indexB = cmdOrder.indexOf(b);
    return (indexA === -1 ? 999 : indexA) - (indexB === -1 ? 999 : indexB);
  });

  return (
    <CommonLayout>
      <div className="space-y-4">
        <Accordion type="multiple" className="w-full">
          {sortedGroups.map(([cmd, sensors]) => (
            <AccordionItem key={cmd} value={cmd}>
              <AccordionTrigger>{cmd}</AccordionTrigger>
              <AccordionContent>
                <div className="grid grid-cols-1 lg:grid-cols-12 gap-4">
                  {sensors.map((sensor, index) => (
                    <LiveCellVoltage
                      key={index}
                      data={sensor.value}
                      isCharging={sensor.value > 27}
                    />
                  ))}
                </div>
              </AccordionContent>
            </AccordionItem>
          ))}
        </Accordion>
      </div>
    </CommonLayout>
  );
}

export default BmsMonitor;
