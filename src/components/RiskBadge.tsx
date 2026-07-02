import { RiskLevel, RISK_COLOR } from "../types/explained";

export default function RiskBadge({ level }: { level: RiskLevel }) {
  return (
    <span
      className={`inline-flex items-center gap-1.5 rounded-full px-2.5 py-0.5 text-xs font-medium text-neutral-950 ${RISK_COLOR[level]}`}
    >
      {level}
    </span>
  );
}
