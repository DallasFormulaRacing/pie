import { useDaqSocket } from "@/hooks/useDaqSocket";
import { CommonLayout } from "@/components/CommonLayout";
import { NodeStatus } from "@/components/NodeStatus";
import { LiveTestGraph } from "@/components/Graph";
import { Button } from "@/components/ui/button";
import { RefreshCw } from "lucide-react";
import type { UiCommand } from "@/types/daq";
export function DaqMonitor() {
  const { nodes, sendCommand } = useDaqSocket(`ws://${window.location.hostname}:8080`);
  const nodeNames = ["FL_NODE", "FR_NODE", "RL_NODE", "RR_NODE", "NUC_1", "NUC_2", "PDM_01"];

  return (
    <CommonLayout>
      {/* Main Grid: 5 columns total for a 4:1 split */}
      <div className="grid grid-cols-1 xl:grid-cols-5 gap-8">
        
        {/* Left Column: Telemetry Graphs (80% width on XL) */}
        <div className="xl:col-span-4 grid grid-cols-1 lg:grid-cols-2 gap-4">
          <LiveTestGraph title="Wheel Speed" description="Average Wheel Speed (km/h)" />
          <LiveTestGraph title="Lin Pot" description="Linear Potentiometer Travel (mm)" />
          <LiveTestGraph title="Inverter Temp" description="Inverter Temperature (°C)" />
          <LiveTestGraph title="Tire Temperature" description="Average Tire Temperature (°C)" />
        </div>

        {/* Right Column: Node Management Sidebar (20% width on XL) */}
        <div className="xl:col-span-1 flex flex-col gap-4">
          <div className="flex items-center justify-between">
            <h3 className="text-sm font-semibold text-muted-foreground uppercase tracking-wider">
              System Nodes
            </h3>
            <Button 
              variant="ghost" 
              size="icon" 
              onClick={() => sendCommand({ command: "REFRESH_NODES"})}
            >
              <RefreshCw className="h-4 w-4" />
            </Button>
          </div>
                    
          <div className="grid grid-cols-2 xl:grid-cols-1 gap-2">
            {nodeNames.map((name) => (
              <NodeStatus 
                key={name}
                name={name} 
                isOnline={!!nodes[name]} 
                onCommand={(cmd, payload) => 
                      sendCommand({ 
                        command: cmd, 
                        payload: { node: name, ...payload } 
                      } as UiCommand) 
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