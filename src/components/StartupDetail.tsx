import { useState } from "react";
import { ExplainedPersistence, SOURCE_LABEL } from "../types/explained";
import RiskBadge from "./RiskBadge";

export default function StartupDetail({ entry }: { entry: ExplainedPersistence | null }) {
  const [copied, setCopied] = useState(false);

  if (!entry) {
    return (
      <div className="flex h-full items-center justify-center text-neutral-500 text-sm">
        Select a startup entry to see why it's flagged.
      </div>
    );
  }

  const copyCommand = async () => {
    try {
      await navigator.clipboard.writeText(entry.command);
      setCopied(true);
      setTimeout(() => setCopied(false), 1200);
    } catch {
      // Convenience feature only - not worth surfacing an error if it fails.
    }
  };

  return (
    <div className="p-4 space-y-4">
      <div>
        <div className="flex items-center gap-2">
          <h2 className="text-lg font-semibold">{entry.name}</h2>
          <RiskBadge level={entry.riskLevel} />
        </div>
        <p className="text-sm text-neutral-400">{SOURCE_LABEL[entry.source]}</p>
        <button
          onClick={copyCommand}
          title="Click to copy"
          className="text-xs text-neutral-500 hover:text-neutral-300 break-all mt-1 text-left"
        >
          {entry.command} {copied && <span className="text-risk-green">(copied)</span>}
        </button>
      </div>

      <div className="rounded-md border border-neutral-800 bg-neutral-900 p-3">
        <p className="text-sm">{entry.summary}</p>
      </div>

      {entry.explanations.length > 0 && (
        <div>
          <h3 className="text-xs uppercase tracking-wide text-neutral-500 mb-2">
            Why this is flagged
          </h3>
          <ul className="space-y-1.5 text-sm text-neutral-300">
            {entry.explanations.map((line, i) => (
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

