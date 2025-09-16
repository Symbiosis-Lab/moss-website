// Command Registry - Command definitions and execution
// Manages the registry of available commands and their execution

export interface Command {
  action: string;
  name: string;
  section: string;
  icon?: string;
}

export class CommandRegistry {
  private commands: Map<string, Command>;

  constructor() {
    this.commands = new Map<string, Command>();
    this.registerDefaultCommands();
  }

  /**
   * Register a new command
   */
  register(command: Command): void {
    this.commands.set(command.action, command);
    console.log(`📝 Registered command: ${command.action}`);
  }

  /**
   * Get a command by action
   */
  get(action: string): Command | undefined {
    return this.commands.get(action);
  }

  /**
   * Get all commands
   */
  getAll(): Command[] {
    return Array.from(this.commands.values());
  }

  /**
   * Check if command exists
   */
  has(action: string): boolean {
    return this.commands.has(action);
  }

  /**
   * Remove a command
   */
  unregister(action: string): boolean {
    const success = this.commands.delete(action);
    if (success) {
      console.log(`🗑️ Unregistered command: ${action}`);
    }
    return success;
  }

  /**
   * Search commands by name or section
   */
  search(query: string): Command[] {
    const lowercaseQuery = query.toLowerCase();
    return this.getAll().filter(cmd =>
      cmd.name.toLowerCase().includes(lowercaseQuery) ||
      cmd.section.toLowerCase().includes(lowercaseQuery)
    );
  }

  /**
   * Get commands grouped by section
   */
  getBySection(): Map<string, Command[]> {
    const sections = new Map<string, Command[]>();

    for (const command of this.commands.values()) {
      if (!sections.has(command.section)) {
        sections.set(command.section, []);
      }
      sections.get(command.section)!.push(command);
    }

    return sections;
  }

  /**
   * Register default commands
   */
  private registerDefaultCommands(): void {
    this.register({
      action: "refresh-preview",
      name: "Refresh Preview",
      section: "Settings",
      icon: "🔄",
    });

    this.register({
      action: "setup-git",
      name: "Setup Git Repository",
      section: "Publish",
      icon: "⚙️",
    });

    this.register({
      action: "compile-folder",
      name: "Compile Folder",
      section: "Build",
      icon: "📁",
    });

    this.register({
      action: "show-system-info",
      name: "System Information",
      section: "Settings",
      icon: "ℹ️",
    });

    this.register({
      action: "install-finder-integration",
      name: "Install Finder Integration",
      section: "Settings",
      icon: "🔧",
    });

    console.log(`✅ Registered ${this.commands.size} default commands`);
  }
}