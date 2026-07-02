import { ExplainedProcess } from "../types/explained";
import RiskBadge from "./RiskBadge";

export default function ProcessDetail({ process }: { process: ExplainedProcess | null }) {
  if (!process) {
    return (
      <div className="flex h-full items-center justify-center text-neutral-500 text-sm">
        Select a process to see why it's flagged.
      </div>
    );
  }

  return (
    <div className="p-4 space-y-4">
      <div>
        <div className="flex items-center gap-2">
          <h2 className="text-lg font-semibold">{process.name}</h2>
          <RiskBadge level={process.riskLevel} />
        </div>
        <p className="text-sm text-neutral-400">{process.exePath}</p>
      </div>

      <div className="rounded-md border border-neutral-800 bg-neutral-900 p-3">
        <p className="text-sm">{process.summary}</p>
      </div>

      {process.explanations.length > 0 && (
        <div>
          <h3 className="text-xs uppercase tracking-wide text-neutral-500 mb-2">
            Why this is flagged
          </h3>
          <ul className="space-y-1.5 text-sm text-neutral-300">
            {process.explanations.map((line, i) => (
              <li key={i} className="flex gap-2">
                <span className="text-neutral-600">-</span>
                <span>{line}</span>
              </li>
            ))}
          </ul>
        </div>
      )}
    </div>
  );
}
