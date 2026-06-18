import { CommonLayout } from "@/components/CommonLayout";
import { useDaqSocket } from "@/hooks/backendSocket";
import { useState, useEffect } from "react";
import type { SensorReading } from "@/types/backend";

const WS_URL = `ws://localhost:9002`;

const SEGMENTS = ["Segment 1", "Segment 2", "Segment 3", "Segment 4", "Segment 5", "Segment 6"];

interface SegmentData {
  cells: SensorReading[];
  temps: SensorReading[];
}

export function BmsMonitor() {
  const { data } = useDaqSocket(WS_URL);
  
  const [segments, setSegments] = useState<Record<string, SegmentData>>({
    "Segment 1": { cells: [], temps: [] },
    "Segment 2": { cells: [], temps: [] },
    "Segment 3": { cells: [], temps: [] },
    "Segment 4": { cells: [], temps: [] },
    "Segment 5": { cells: [], temps: [] },
    "Segment 6": { cells: [], temps: [] },
  });

  useEffect(() => {
    if (!data?.cmd || !data.sensors) return;

    setSegments(prev => {
      const updated = { ...prev };
      switch (data.cmd) {
        case "First 24 Cells":   updated["Segment 1"].cells = data.sensors; break;
        case "Second 24 Cells":  updated["Segment 2"].cells = data.sensors; break;
        case "Third 24 Cells":   updated["Segment 3"].cells = data.sensors; break;
        case "Fourth 24 Cells":  updated["Segment 4"].cells = data.sensors; break;
        case "Fifth 24 Cells":   updated["Segment 5"].cells = data.sensors; break;
        case "Sixth 24 Cells":   updated["Segment 6"].cells = data.sensors; break;
        case "First 60 Temps":
          updated["Segment 1"].temps = data.sensors.slice(0, 20);
          updated["Segment 2"].temps = data.sensors.slice(20, 40);
          updated["Segment 3"].temps = data.sensors.slice(40, 60);
          break;
        case "Last 60 Temps":
          updated["Segment 4"].temps = data.sensors.slice(0, 20);
          updated["Segment 5"].temps = data.sensors.slice(20, 40);
          updated["Segment 6"].temps = data.sensors.slice(40, 60);
          break;
      }
      return updated;
    });
  }, [data]);

  return (
    <CommonLayout>
      <div className="h-[calc(100vh-4rem)] grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 xl:grid-rows-3 gap-3 bg-zinc-950 p-3 text-zinc-200 font-mono select-none box-border">
        {SEGMENTS.map((segName) => {
          const { cells, temps } = segments[segName];

          return (
            <div key={segName} className="border border-zinc-800 bg-zinc-900/20 rounded-lg p-3 flex flex-col justify-between overflow-hidden shadow-md min-h-0">
              
              <div className="flex justify-between items-center border-b border-zinc-800 pb-1.5 mb-2">
                <span className="text-sm font-black tracking-wider text-orange-500 uppercase">{segName}</span>
              </div>

              <div className="grid grid-cols-1 md:grid-cols-10 gap-4 flex-1 min-h-0 items-center">
                
                {/* --- CELLS SECTION --- */}
                <div className="md:col-span-6 grid grid-cols-3 gap-x-3 gap-y-1">
                  <div className="space-y-0.5">
                    {Array.from({ length: 8 }).map((_, i) => (
                      <DataRow key={i} label={`C${i + 1}`} sensor={cells[i]} unit="V" precision={3} />
                    ))}
                  </div>
                  <div className="space-y-0.5">
                    {Array.from({ length: 8 }).map((_, i) => (
                      <DataRow key={i} label={`C${i + 9}`} sensor={cells[i + 8]} unit="V" precision={3} />
                    ))}
                  </div>
                  <div className="space-y-0.5">
                    {Array.from({ length: 8 }).map((_, i) => (
                      <DataRow key={i} label={`C${i + 17}`} sensor={cells[i + 16]} unit="V" precision={3} />
                    ))}
                  </div>
                </div>

                {/* --- TEMPERATURES SECTION --- */}
                <div className="md:col-span-4 grid grid-cols-2 gap-x-3 gap-y-1 border-t md:border-t-0 md:border-l border-zinc-800 pt-2 md:pt-0 md:pl-3 h-full items-center">
                  <div className="space-y-0.5">
                    {Array.from({ length: 10 }).map((_, i) => (
                      <DataRow key={i} label={`T${i + 1}`} sensor={temps[i]} unit="°" precision={1} isTemp />
                    ))}
                  </div>
                  <div className="space-y-0.5">
                    {Array.from({ length: 10 }).map((_, i) => (
                      <DataRow key={i} label={`T${i + 11}`} sensor={temps[i + 11]} unit="°" precision={1} isTemp />
                    ))}
                  </div>
                </div>

              </div>
            </div>
          );
        })}
      </div>
    </CommonLayout>
  );
}

interface RowProps {
  label: string;
  sensor?: SensorReading;
  unit: string;
  precision: number;
  isTemp?: boolean;
}

function DataRow({ label, sensor, unit, precision, isTemp }: RowProps) {
  if (!sensor) {
    return (
      <div className="flex justify-between text-zinc-700 text-[13px] leading-tight animate-pulse">
        <span>{label}</span>
        <span>—.——</span>
      </div>
    );
  }

  const val = sensor.value;
  
  let valueClass = "text-zinc-200 font-medium";
  if (isTemp && val > 38) { // temp threshold should be adjusted later
    valueClass = "text-rose-500 font-bold bg-rose-950/30 px-1 rounded";
  } else if (!isTemp && val < 3.7) {
    valueClass = "bg-cyan-500 text-zinc-950 px-1 rounded font-black tracking-tight";
  }

  return (
    <div className="flex justify-between items-center group hover:bg-zinc-800/40 px-1 py-0 rounded transition-colors text-[13px] leading-tight">
      <span className="text-zinc-500 font-bold text-[11px]">{label}</span>
      <span className={valueClass}>
        {val.toFixed(precision)}
        <span className="text-[9px] text-zinc-500 ml-0.5 font-normal">{unit}</span>
      </span>
    </div>
  );
}

export default BmsMonitor;