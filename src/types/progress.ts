// Progress-related type definitions

export interface ProgressStep {
  name: string;
  status: "pending" | "active" | "completed" | "error";
  percentage?: number;
  message?: string;
  startTime?: number;
  endTime?: number;
}

export interface ProgressState {
  currentStep: string;
  steps: ProgressStep[];
  overallPercentage: number;
  isCompleted: boolean;
  hasError: boolean;
  errorMessage?: string;
}

export interface ProgressRenderer {
  renderStep(step: ProgressStep): string;
  renderOverall(state: ProgressState): string;
  renderError(error: string): string;
}

export interface ProgressTracker {
  start(totalSteps: number): void;
  updateStep(stepName: string, status: ProgressStep["status"], message?: string): void;
  complete(): void;
  error(message: string): void;
  getState(): ProgressState;
  reset(): void;
}