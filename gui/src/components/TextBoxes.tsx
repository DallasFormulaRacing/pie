import { useState } from "react";
import { cn } from "@/lib/utils"
import type { FrontendCommand } from "@/types/backend";

interface TextBoxesProps {
    sendCommand: (cmd: FrontendCommand) => void;
}

export const TextBoxes = ({ sendCommand }: TextBoxesProps) => {
  const [voltage, setVoltage] = useState("");
  const [current, setCurrent] = useState("");
  const [voltageError, setVoltageError] = useState("");
  const [currentError, setCurrentError] = useState("");

  const validateVoltageValue = (value: string, name: string): string => {
    const num = parseFloat(value);
    if (num < 50) return `${name} < 50`;
    if (num > 400) return `${name} > 400`;
    return "";
  };

  const validateCurrentValue = (value: string, name: string): string => {
    const num = parseFloat(value);
    if (num < 1) return `${name} < 1`;
    if (num > 10) return `${name} > 10`;
    return "";
  };

  const handleVoltageChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const value = e.target.value;
    setVoltage(value);
    setVoltageError(validateVoltageValue(value, "Voltage"));
  };

  const handleCurrentChange = (e: React.ChangeEvent<HTMLTextAreaElement>) => {
    const value = e.target.value;
    setCurrent(value);
    setCurrentError(validateCurrentValue(value, "Current"));
  };

  const handleSend = () => {
    const vError = validateVoltageValue(voltage, "Voltage");
    const cError = validateCurrentValue(current, "Current");
    setVoltageError(vError);
    setCurrentError(cError);
    if (!vError && !cError) {
      const v = parseFloat(voltage);
      const c = parseFloat(current);
      sendCommand({ cmd: "setParameters", voltage: v, current: c });
      setVoltage("");
      setCurrent("");
      setVoltageError("");
      setCurrentError("");
    }
  };

  return (
    <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
      
      <div className="flex gap-3">
        <div className="flex flex-col">
          <label className="text-[10px] uppercase font-bold tracking-tight text-blue-400">
            Voltage (V)
          </label>
          <input
            type="text"
            placeholder="Enter Voltage"
            value={voltage}
            onChange={handleVoltageChange}
            className={cn(
              "px-2 py-3 rounded text-sm font-mono border resize-none w-40",
              "bg-card text-card-foreground",
              "outline-none transition-all duration-200",
              voltageError ? "border-red-500" : voltage && "border-green-500"
            )}
          />
          {voltageError && <p className="text-red-500 text-xs">{voltageError}</p>}
        </div>
      
        <div className="flex flex-col">
          <label className="text-[10px] uppercase font-bold tracking-tight text-blue-400">
            Current (A)
          </label>
          <textarea
            placeholder="Enter Current"
            value={current}
            onChange={handleCurrentChange}
            className={cn(
              "px-2 py-2 rounded text-sm font-mono border resize-none w-40",
              "bg-card text-card-foreground",
              "outline-none transition-all duration-200",
              currentError ? "border-red-500" : current && "border-green-500"
            )}
          />
          {currentError && <p className="text-red-500 text-xs">{currentError}</p>}
        </div>
        <div className="flex flex-col justify-center">
          <button
            onClick={handleSend}
            className="h-[38px] px-4 py-2 bg-blue-600 text-white text-sm font-bold rounded hover:bg-blue-500 transition-colors"
          >
            SEND PARAMETERS
          </button>
        </div>
      </div>
    </div>
  )
}