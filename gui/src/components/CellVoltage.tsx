import { cn } from "@/lib/utils"

interface CVProps {
    data?: number;
    isCharging?: boolean;
}

export const LiveCellVoltage = ({ data = 0, isCharging = false }: CVProps) => {
  return (
    <div
      className={cn(
        "px-2 py-1 rounded text-sm font-mono",
        isCharging ? "bg-green-600 text-white" : "text-white"
      )}
    >
      {data.toFixed(3)} V
    </div>
  );
}
