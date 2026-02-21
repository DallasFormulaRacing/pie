import { useState } from "react"
import { Button } from "@/components/ui/button"
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu"
import { cn } from "@/lib/utils"
import { 
  RefreshCw, 
  Lightbulb, 
  LightbulbOff,
  RotateCw, 
  Cpu, 
  Activity 
} from "lucide-react"
import type { UiCommand } from "@/types/daq"
interface NodeStatusProps {
  name: string;
  isOnline: boolean;
  onCommand: (cmd: UiCommand['command'], payload?: Record<string, unknown>) => void;
}

export function NodeStatus({ name, isOnline, onCommand }: NodeStatusProps) {
  const [ledOn, setLedOn] = useState(false);

    const handleToggleLed = () => {
        const newState = !ledOn;
        setLedOn(newState);
        onCommand("TOGGLE_LED", { state: newState }); 
    }

  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button 
          variant="outline" 
          className="w-full justify-between h-auto py-3 px-4 font-mono text-xs group"
        >
          <div className="flex items-center gap-3">
            <span className="relative flex h-2 w-2">
              {isOnline && (
                <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-green-400 opacity-75"></span>
              )}
              <span className={cn(
                "relative inline-flex rounded-full h-2 w-2",
                isOnline ? "bg-green-500" : "bg-slate-400"
              )}></span>
            </span>
            <span className={cn(!isOnline && "text-muted-foreground")}>{name}</span>
          </div>
          <Activity className={cn(
            "h-3 w-3 transition-opacity",
            isOnline ? "opacity-30 group-hover:opacity-100" : "opacity-10"
          )} />
        </Button>
      </DropdownMenuTrigger>
      
      <DropdownMenuContent align="end" className="w-52">
        <div className="px-2 py-1.5 text-[10px] font-bold uppercase text-muted-foreground">
          {isOnline ? "Node Controls" : "Node Offline"}
        </div>
        <DropdownMenuSeparator />
        
        <DropdownMenuItem onClick={() => onCommand("PING_NODE")}>
          <RefreshCw className="mr-2 h-4 w-4" />
          <span>Ping Node</span>
        </DropdownMenuItem>
        
        <DropdownMenuItem 
          disabled={!isOnline} 
          onClick={handleToggleLed}
        >
          {ledOn ? (
            <>
              <Lightbulb className="mr-2 h-4 w-4 text-yellow-400 fill-yellow-400" />
              <span>Turn LED Off</span>
            </>
          ) : (
            <>
              <LightbulbOff className="mr-2 h-4 w-4 text-muted-foreground" />
              <span>Turn LED On</span>
            </>
          )}
        </DropdownMenuItem>

        <DropdownMenuSeparator />
        
        <DropdownMenuItem 
          disabled={!isOnline} 
          onClick={() => onCommand("REBOOT")}
        >
          <RotateCw className="mr-2 h-4 w-4" />
          <span>Reboot</span>
        </DropdownMenuItem>
        
        <DropdownMenuItem 
          disabled={!isOnline}
          variant="destructive"
          onClick={() => onCommand("BOOTLOADER")}
        >
          <Cpu className="mr-2 h-4 w-4" />
          <span>Jump to Bootloader</span>
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  )
}