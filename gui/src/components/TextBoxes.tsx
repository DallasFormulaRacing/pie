import * as React from "react"
import { cn } from "@/lib/utils"

interface CVProps {
    data?: number;
    isCharging?: boolean;
    callback: CallableFunction
}

// callback in interface needs to be done differently 
export const TextBoxes = ( ) => {
  return (
    <div className="grid grid-cols-1 lg:grid-cols-4 gap-4">
      
      <div className="flex gap-10">
        <label className="text-[10px] uppercase font-bold tracking-tight text-blue-400">
          Voltage
        </label>
        <input
          type="text"
          placeholder="Enter Voltage"
          className={cn(
            "px-2 py-1 rounded text-sm font-mono border resize-none",
            "bg-card text-card-foreground",
            "outline-none transition-all duration-200",

            "focus:ring-2 focus:ring-blue-400",
            "hover:border-blue-400"
          )}
        />
      
        <label className="text-[10px] uppercase font-bold tracking-tight text-blue-400">
          Current
        </label>
        <textarea
          placeholder="Enter Current"
          className={cn(
            "px-2 py-1 rounded text-sm font-mono border resize-none",
            "bg-card text-card-foreground",
            "outline-none transition-all duration-200",

            "focus:ring-2 focus:ring-blue-400",
            "hover:border-blue-400"
          )}
        />
      </div>

    </div>
  )
}