import * as React from "react"
import { cn } from "@/lib/utils"

interface TBProps {
  callback: (voltage: string, current: string) => void
}

export const TextBoxes = ({ callback }: TBProps) => {
  const [voltage, setVoltage] = React.useState("")
  const [current, setCurrent] = React.useState("")

  const handleSubmit = () => {
    callback(voltage, current)
  }

  return (
    <div className="grid grid-cols-1 lg:grid-cols-4 gap-4">
      <div className="flex gap-10 items-end">

        <div className="flex flex-col gap-1">
          <label className="text-[10px] uppercase font-bold tracking-tight text-blue-400">
            Voltage
          </label>
          <input
            type="text"
            value={voltage}
            onChange={(e) => setVoltage(e.target.value)}
            placeholder="Enter Voltage"
            className={cn(
              "px-2 py-1 rounded text-sm font-mono border",
              "bg-card text-card-foreground",
              "outline-none transition-all duration-200",
              "focus:ring-2 focus:ring-blue-400",
              "hover:border-blue-400"
            )}
          />
        </div>

        <div className="flex flex-col gap-1">
          <label className="text-[10px] uppercase font-bold tracking-tight text-blue-400">
            Current
          </label>
          <input
            type="text"
            value={current}
            onChange={(e) => setCurrent(e.target.value)}
            placeholder="Enter Current"
            className={cn(
              "px-2 py-1 rounded text-sm font-mono border",
              "bg-card text-card-foreground",
              "outline-none transition-all duration-200",
              "focus:ring-2 focus:ring-blue-400",
              "hover:border-blue-400"
            )}
          />
        </div>

        <button
          onClick={handleSubmit}
          className="px-3 py-1 rounded bg-blue-500 text-white text-sm font-semibold hover:bg-blue-600 transition"
        >
          Submit
        </button>

      </div>
    </div>
  )
}