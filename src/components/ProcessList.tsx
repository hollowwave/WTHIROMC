import { ExplainedProcess, RISK_ORDER } from "../types/explained";
import RiskBadge from "./RiskBadge";

interface Props {
  processes: ExplainedProcess[];
  selectedPid: number | null;
  onSelect: (pid: number) => void;
}

function formatMemory(bytes: number): string {
  const mb = bytes / (1024 * 1024);
  return mb >= 1024 ? `${(mb / 1024).toFixed(1)} GB` : `${mb.toFixed(0)} MB`;
}

export default function ProcessList({ processes, selectedPid, onSelect }: Props) {
  const sorted = [...processes].sort(
    (a, b) => RISK_ORDER.indexOf(a.riskLevel) - RISK_ORDER.indexOf(b.riskLevel)
  );

  return (
    <table className="w-full text-sm">
      <thead className="sticky top-0 bg-neutral-950 border-b border-neutral-800 text-xs text-neutral-500 uppercase tracking-wide">
        <tr>
          <th className="text-left font-medium px-4 py-2">Name</th>
          <th className="text-left font-medium px-4 py-2">Publisher</th>
          <th className="text-right font-medium px-4 py-2">CPU</th>
          <th className="text-right font-medium px-4 py-2">Memory</th>
          <th className="text-right font-medium px-4 py-2">Risk</th>
        </tr>
      </thead>
      <tbody className="divide-y divide-neutral-900">
        {sorted.map((p) => {
          const isGreen = p.riskLevel === "Green";
          return (
            <tr
              key={p.pid}
              onClick={() => onSelect(p.pid)}
              className={`cursor-pointer hover:bg-neutral-900 transition-colors ${
                selectedPid === p.pid ? "bg-neutral-900" : ""
              } ${isGreen ? "text-neutral-500" : "text-neutral-100"}`}
            >
              <td className="px-4 py-2 truncate max-w-[180px]">
                <span className="inline-flex items-center gap-1.5">
                  {p.name}
                  {p.isNew && (
                    <span
                      title="Wasn't running the last time you checked"
                      className="shrink-0 text-[10px] font-semibold uppercase tracking-wide text-blue-400 border border-blue-400/40 rounded px-1"
                    >
                      New
                    </span>
                  )}
                </span>
              </td>
              <td className="px-4 py-2 truncate max-w-[160px]">
                {p.publisher ?? <span className="text-neutral-600">Unknown</span>}
              </td>
              <td className="px-4 py-2 text-right tabular-nums">{p.cpuUsage.toFixed(0)}%</td>
              <td className="px-4 py-2 text-right tabular-nums">{formatMemory(p.memoryBytes)}</td>
              <td className="px-4 py-2 text-right">
                <RiskBadge level={p.riskLevel} />
              </td>
            </tr>
          );
        })}
      </tbody>
    </table>
  );
}

