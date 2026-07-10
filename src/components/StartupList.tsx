import { ExplainedPersistence, RISK_ORDER, SOURCE_LABEL } from "../types/explained";
import RiskBadge from "./RiskBadge";

interface Props {
  entries: ExplainedPersistence[];
  selectedKey: string | null;
  onSelect: (key: string) => void;
}

export function entryKey(e: ExplainedPersistence): string {
  return `${e.source}:${e.name}`;
}

export default function StartupList({ entries, selectedKey, onSelect }: Props) {
  const sorted = [...entries].sort(
    (a, b) => RISK_ORDER.indexOf(a.riskLevel) - RISK_ORDER.indexOf(b.riskLevel)
  );

  return (
    <table className="w-full text-sm">
      <thead className="sticky top-0 bg-neutral-950 border-b border-neutral-800 text-xs text-neutral-500 uppercase tracking-wide">
        <tr>
          <th className="text-left font-medium px-4 py-2">Name</th>
          <th className="text-left font-medium px-4 py-2">Source</th>
          <th className="text-left font-medium px-4 py-2">Publisher</th>
          <th className="text-right font-medium px-4 py-2">Risk</th>
        </tr>
      </thead>
      <tbody className="divide-y divide-neutral-900">
        {sorted.map((e) => {
          const isGreen = e.riskLevel === "Green";
          const key = entryKey(e);
          return (
            <tr
              key={key}
              onClick={() => onSelect(key)}
              className={`cursor-pointer hover:bg-neutral-900 transition-colors ${
                selectedKey === key ? "bg-neutral-900" : ""
              } ${isGreen ? "text-neutral-500" : "text-neutral-100"}`}
            >
              <td className="px-4 py-2 truncate max-w-[220px]">
                <span className="inline-flex items-center gap-1.5">
                  {e.name}
                  {e.isNew && (
                    <span
                      title="Wasn't set to run automatically the last time you checked"
                      className="shrink-0 text-[10px] font-semibold uppercase tracking-wide text-blue-400 border border-blue-400/40 rounded px-1"
                    >
                      New
                    </span>
                  )}
                </span>
              </td>
              <td className="px-4 py-2 text-neutral-400">{SOURCE_LABEL[e.source]}</td>
              <td className="px-4 py-2 truncate max-w-[160px]">
                {e.publisher ?? <span className="text-neutral-600">Unknown</span>}
              </td>
              <td className="px-4 py-2 text-right">
                <RiskBadge level={e.riskLevel} />
              </td>
            </tr>
          );
        })}
      </tbody>
    </table>
  );
}

