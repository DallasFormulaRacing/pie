import { cn } from "@/lib/utils"

interface InfoProps {
  title: string;
  inputData?: number;
}

export const InfoBox = ({ title, inputData = 0 }: InfoProps) => {
  return (
    <div
      className={cn(
        "flex flex-col gap-1",
        "w-[24vw] h-[15vh]",
        "transition-all duration-300"
      )}
    >
      <span className="text-[10px] uppercase font-bold tracking-tight text-blue-400">
        {title}
      </span>

      <div
        className={cn(
          "px-2 py-1 rounded text-sm font-mono border bg-card text-card-foreground",
          "flex-1 flex items-center justify-center",
          "transition-all duration-300",

          "hover:shadow-[0_0_10px_rgba(59,130,246,0.8)]", // blue glow
          "hover:border-blue-400"
        )}
      >
        {inputData.toFixed(2)} V
      </div>
    </div>
  );
}