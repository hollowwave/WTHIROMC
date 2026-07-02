export type RiskLevel = "Green" | "Yellow" | "Orange" | "Red" | "Black";

export interface ExplainedProcess {
  pid: number;
  name: string;
  exePath: string;
  publisher: string | null;
  cpuUsage: number;
  memoryBytes: number;
  riskLevel: RiskLevel;
  score: number;
  summary: string;
  explanations: string[];
}

export const RISK_ORDER: RiskLevel[] = ["Black", "Red", "Orange", "Yellow", "Green"];

export const RISK_COLOR: Record<RiskLevel, string> = {
  Green: "bg-risk-green",
  Yellow: "bg-risk-yellow",
  Orange: "bg-risk-orange",
  Red: "bg-risk-red",
  Black: "bg-risk-black",
};
