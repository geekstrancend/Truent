/**
 * TypeScript type definitions for @truent/cli
 */

/**
 * Violation object from Truent report.
 */
export interface Violation {
  id: string;
  severity: "Critical" | "High" | "Medium" | "Low";
  title: string;
  location: string | null;
  message: string;
  recommendation: string;
  cwe: string | null;
  references: string[];
}

/**
 * Passed check object.
 */
export interface PassedCheck {
  id: string;
  title: string;
}

/**
 * Suppressed check object.
 */
export interface SuppressedCheck {
  id: string;
  location: string | null;
  reason: string | null;
}

/**
 * Summary of Truent report.
 */
export interface ReportSummary {
  total_checks: number;
  violations: number;
  critical: number;
  high: number;
  medium: number;
  low: number;
  passed: number;
  suppressed: number;
}

/**
 * Complete Truent report object.
 */
export interface TruentReport {
  version: string;
  timestamp: string;
  chain: string;
  target: string;
  duration_ms: number;
  summary: ReportSummary;
  violations: Violation[];
  passed: PassedCheck[];
  suppressed: SuppressedCheck[];
}

/**
 * Doctor result object.
 */
export interface DoctorResult {
  status: "healthy" | "warning" | "error";
  components: {
    [key: string]: {
      status: "ok" | "warning" | "error";
      message?: string;
    };
  };
}

/**
 * Analyze options.
 */
export interface AnalyzeOptions {
  path: string;
  chain: "evm" | "solana" | "move";
  failOn?: "low" | "medium" | "high" | "critical";
  config?: string;
  verbose?: boolean;
}

/**
 * Run a Truent analysis programmatically.
 */
export function analyze(options: AnalyzeOptions): Promise<TruentReport>;

/**
 * Run truent doctor and return component health status.
 */
export function doctor(): Promise<DoctorResult>;

/**
 * Initialize a .truent.toml in the given directory.
 */
export function init(directory: string): Promise<void>;

/**
 * Get the version of the installed Truent binary.
 */
export function version(): Promise<string>;

/**
 * Check if the Truent binary is installed and working.
 */
export function isInstalled(): Promise<boolean>;
