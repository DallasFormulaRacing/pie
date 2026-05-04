import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";
import { Activity } from "lucide-react";
import type { DeviceStatus } from "@/types/backend";

interface NodeStatusProps {
  device: DeviceStatus;
}

export function NodeStatus({ device }: NodeStatusProps) {
  const { name, online, lastSeenMsAgo } = device;
  const modeLabel = online ? "Online" : "Offline";
  const dotColor = online ? "bg-green-500" : "bg-slate-400";

  return (
    <Button
      variant="outline"
      className="w-full justify-between h-auto py-3 px-4 font-mono text-xs group"
    >
      <div className="flex min-w-0 items-center gap-3">
        <span className="relative flex h-2 w-2 shrink-0">
          {online && (
            <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-green-400 opacity-75" />
          )}
          <span className={cn("relative inline-flex rounded-full h-2 w-2", dotColor)} />
        </span>
        <span className={cn("truncate", !online && "text-muted-foreground")}>
          {name}
        </span>
      </div>
      <div className="flex shrink-0 items-center gap-2">
        <span
          className={cn(
            "text-[10px] uppercase",
            online ? "text-muted-foreground" : "text-muted-foreground/50",
          )}
        >
          {lastSeenMsAgo !== null ? `${lastSeenMsAgo}ms` : modeLabel}
        </span>
        <Activity
          className={cn(
            "h-3 w-3 transition-opacity",
            online ? "opacity-30 group-hover:opacity-100" : "opacity-10",
          )}
        />
      </div>
    </Button>
  );
}
