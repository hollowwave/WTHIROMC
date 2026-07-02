import { ExplainedProcess, RISK_ORDER } from "../types/explained";
import RiskBadge from "./RiskBadge";

interface Props {
  processes: ExplainedProcess[];
  selectedPid: number | null;
  onSelect: (pid: number) => void;
}

export default function ProcessList({ processes, selectedPid, onSelect }: Props) {
  const sorted = [...processes].sort(
    (a, b) => RISK_ORDER.indexOf(a.riskLevel) - RISK_ORDER.indexOf(b.riskLevel)
  );

  return (
    <div className="divide-y divide-neutral-800">
      {sorted.map((p) => (
        <button
          key={p.pid}
          onClick={() => onSelect(p.pid)}
          className={`w-full text-left px-4 py-2.5 flex items-center justify-between hover:bg-neutral-900 transition-colors ${
            selectedPid === p.pid ? "bg-neutral-900" : ""
          }`}
        >
          <div className="min-w-0">
            <p className="text-sm font-medium truncate">{p.name}</p>
            <p className="text-xs text-neutral-500 truncate">
              {p.publisher ?? "Unknown publisher"} - PID {p.pid}
            </p>
          </div>
          <RiskBadge level={p.riskLevel} />
        </button>
      ))}
    </div>
  );
}
