import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { cn } from "@/lib/utils";
import { RefreshCw, RotateCw, Activity } from "lucide-react";
import type { DeviceStatus } from "@/types/backend";

interface NodeStatusProps {
  device: DeviceStatus;
  onPing: (deviceId: number) => void;
  onReboot: (deviceId: number) => void;
}

export function NodeStatus({ device, onPing, onReboot }: NodeStatusProps) {
  const { deviceId, name, online, mode } = device;

  const modeLabel = online
    ? mode === "bootloader"
      ? "Bootloader"
      : "App"
    : "Offline";

  const dotColor = online
    ? mode === "bootloader"
      ? "bg-yellow-500"
      : "bg-green-500"
    : "bg-slate-400";

  const pingColor = online
    ? mode === "bootloader"
      ? "bg-yellow-400"
      : "bg-green-400"
    : "";

  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button
          variant="outline"
          className="w-full justify-between h-auto py-3 px-4 font-mono text-xs group"
        >
          <div className="flex items-center gap-3">
            <span className="relative flex h-2 w-2">
              {online && (
                <span
                  className={cn(
                    "animate-ping absolute inline-flex h-full w-full rounded-full opacity-75",
                    pingColor,
                  )}
                />
              )}
              <span
                className={cn(
                  "relative inline-flex rounded-full h-2 w-2",
                  dotColor,
                )}
              />
            </span>
            <span className={cn(!online && "text-muted-foreground")}>
              {name}
            </span>
          </div>
          <div className="flex items-center gap-2">
            <span
              className={cn(
                "text-[10px] uppercase",
                online ? "text-muted-foreground" : "text-muted-foreground/50",
              )}
            >
              {modeLabel}
            </span>
            <Activity
              className={cn(
                "h-3 w-3 transition-opacity",
                online ? "opacity-30 group-hover:opacity-100" : "opacity-10",
              )}
            />
          </div>
        </Button>
      </DropdownMenuTrigger>

      <DropdownMenuContent align="end" className="w-52">
        <div className="px-2 py-1.5 text-[10px] font-bold uppercase text-muted-foreground">
          {name} — {modeLabel}
        </div>
        <DropdownMenuSeparator />

        <DropdownMenuItem onClick={() => onPing(deviceId)}>
          <RefreshCw className="mr-2 h-4 w-4" />
          <span>Ping</span>
        </DropdownMenuItem>

        <DropdownMenuSeparator />

        <DropdownMenuItem disabled={!online} onClick={() => onReboot(deviceId)}>
          <RotateCw className="mr-2 h-4 w-4" />
          <span>Reboot</span>
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
