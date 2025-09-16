// Global TypeScript Interfaces and Types
// Centralized type definitions for the moss application

// Re-export types from bindings for convenience
export type { ProgressUpdate } from "../bindings";

// App Configuration Types
export interface MossCompileConfig {
  folder_path: string;
  auto_serve: boolean;
}

// Preview System Types
export interface PreviewManager {
  initialize(): Promise<void>;
  startCompilation(folderPath: string, autoServe?: boolean): Promise<void>;
  loadPreview(url: string): void;
  showWelcomeMessage(): void;
  showMessage(message: string, isLoading?: boolean, details?: any): void;
  showProgress(message: string, percentage: number, isActive: boolean): void;
  refresh(): void;
  getIframe(): HTMLIFrameElement;
}

// Panel System Types
export interface Command {
  action: string;
  name: string;
  section: string;
  icon?: string;
}

export type ButtonState = "default" | "loading" | "success" | "error" | "active";

export interface ActionContext {
  button: HTMLElement;
  setButtonState: (button: HTMLElement, state: ButtonState) => void;
  previewManager?: PreviewManager;
}

export interface CommandRegistry {
  register(command: Command): void;
  get(action: string): Command | undefined;
  getAll(): Command[];
  has(action: string): boolean;
  unregister(action: string): boolean;
  search(query: string): Command[];
  getBySection(): Map<string, Command[]>;
}

export interface ActionHandlers {
  register(action: string, handler: (context: ActionContext) => Promise<void>): void;
  execute(action: string, context: ActionContext): Promise<void>;
  has(action: string): boolean;
}

export interface KeyboardShortcut {
  key: string;
  description: string;
  action?: string;
}

export interface KeyboardShortcutHandler {
  showPanel: () => void;
  hidePanel: () => void;
  togglePanel: () => void;
  isPanelVisible: () => boolean;
  showCommandPalette: () => void;
  hideCommandPalette: () => void;
  toggleCommandPalette: () => void;
  isCommandPaletteVisible: () => boolean;
  executeCommand: (action: string) => void;
}

export interface PanelManager extends KeyboardShortcutHandler {
  getCommandRegistry(): CommandRegistry;
  getActionHandlers(): ActionHandlers;
}

// Backend Integration Types
export interface TauriCommandResult<T> {
  status: "ok" | "error";
  data?: T;
  error?: string;
}

export interface SystemInfo {
  os: string;
  finder_integration: boolean;
  app_version: string;
}

// UI Utility Types
export type NotificationType = "info" | "success" | "error";

export interface UIUtils {
  showNotification(message: string, type?: NotificationType, duration?: number): void;
  createSpinner(size?: number): HTMLElement;
  getElement<T extends Element>(selector: string, required?: boolean): T | null;
  getElementById<T extends HTMLElement>(id: string, required?: boolean): T | null;
  debounce<T extends (...args: any[]) => any>(func: T, wait: number): (...args: Parameters<T>) => void;
  throttle<T extends (...args: any[]) => any>(func: T, limit: number): (...args: Parameters<T>) => void;
}

// App Types
export interface MossApp {
  initialize(): Promise<void>;
  getPreviewManager(): PreviewManager | null;
  getPanelManager(): PanelManager | null;
  isReady(): boolean;
  destroy(): void;
}

// Global Window Extensions
declare global {
  interface Window {
    tauriInvoke: any; // Import from bindings
    mossPanelSystem: PanelManager;
    mossPreviewManager: PreviewManager;
    __MOSS_COMPILE_CONFIG__?: MossCompileConfig;
  }
}

// Template Types
export interface ProgressTemplates {
  loading(message: string, percentage: number): string;
  completed(message: string, percentage: number): string;
  welcome(message: string): string;
  withSteps(message: string, steps: any[]): string;
}

// Event Types
export interface MossEvent {
  type: string;
  data?: any;
  timestamp: number;
}

// Progress Step Types
export interface ProgressStep {
  name: string;
  status: "pending" | "active" | "completed" | "error";
}

// Error Types
export interface MossError extends Error {
  code?: string;
  context?: any;
}

// Configuration Types
export interface MossConfig {
  preview: {
    defaultAutoServe: boolean;
    timeouts: {
      compilation: number;
      serverStart: number;
    };
  };
  panel: {
    autoHideDelay: number;
    sections: {
      [key: string]: {
        collapsed: boolean;
      };
    };
  };
  keyboard: {
    enabled: boolean;
    shortcuts: {
      [key: string]: string;
    };
  };
}

// Utility Types
export type DeepPartial<T> = {
  [P in keyof T]?: T[P] extends object ? DeepPartial<T[P]> : T[P];
};

export type Optional<T, K extends keyof T> = Omit<T, K> & Partial<Pick<T, K>>;

export type RequireAtLeastOne<T, Keys extends keyof T = keyof T> = Pick<T, Exclude<keyof T, Keys>> &
  {
    [K in Keys]-?: Required<Pick<T, K>> & Partial<Pick<T, Exclude<Keys, K>>>;
  }[Keys];

// Module Export Types
export * from "./progress";
export * from "./commands";