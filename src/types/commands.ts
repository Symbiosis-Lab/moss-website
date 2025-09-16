// Command-related type definitions

export interface CommandDefinition {
  action: string;
  name: string;
  section: string;
  icon?: string;
  description?: string;
  shortcut?: string;
  enabled?: boolean;
}

export interface CommandContext {
  button?: HTMLElement;
  source: "button" | "keyboard" | "palette" | "api";
  metadata?: Record<string, any>;
}

export interface CommandHandler {
  execute(context: CommandContext): Promise<void> | void;
  canExecute?(context: CommandContext): boolean;
  getDescription?(): string;
}

export interface CommandRegistry {
  register(command: CommandDefinition, handler: CommandHandler): void;
  unregister(action: string): boolean;
  get(action: string): { definition: CommandDefinition; handler: CommandHandler } | undefined;
  getAll(): Array<{ definition: CommandDefinition; handler: CommandHandler }>;
  getBySection(section: string): Array<{ definition: CommandDefinition; handler: CommandHandler }>;
  search(query: string): Array<{ definition: CommandDefinition; handler: CommandHandler }>;
  execute(action: string, context: CommandContext): Promise<void>;
  canExecute(action: string, context: CommandContext): boolean;
}

export interface CommandPalette {
  show(): void;
  hide(): void;
  toggle(): void;
  isVisible(): boolean;
  filter(query: string): void;
  selectNext(): void;
  selectPrevious(): void;
  executeSelected(): void;
}

// Built-in command actions
export const BUILT_IN_COMMANDS = {
  REFRESH_PREVIEW: "refresh-preview",
  SETUP_GIT: "setup-git",
  COMPILE_FOLDER: "compile-folder",
  SHOW_SYSTEM_INFO: "show-system-info",
  INSTALL_FINDER_INTEGRATION: "install-finder-integration",
  TOGGLE_PANEL: "toggle-panel",
  SHOW_COMMAND_PALETTE: "show-command-palette",
  SHOW_HELP: "show-help"
} as const;

export type BuiltInCommand = typeof BUILT_IN_COMMANDS[keyof typeof BUILT_IN_COMMANDS];