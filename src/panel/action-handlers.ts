// Action Handlers - Command execution logic
// Handles the execution of different panel actions

import { TauriCommands } from "../backend/tauri-integration";

export type ButtonState = "default" | "loading" | "success" | "error" | "active";

export interface ActionContext {
  button: HTMLElement;
  setButtonState: (button: HTMLElement, state: ButtonState) => void;
  previewManager?: any; // Reference to preview manager for actions that need it
}

/**
 * Action handler registry
 */
export class ActionHandlers {
  private handlers: Map<string, (context: ActionContext) => Promise<void>>;

  constructor() {
    this.handlers = new Map();
    this.registerDefaultHandlers();
  }

  /**
   * Register an action handler
   */
  register(action: string, handler: (context: ActionContext) => Promise<void>): void {
    this.handlers.set(action, handler);
    console.log(`🎯 Registered action handler: ${action}`);
  }

  /**
   * Execute an action
   */
  async execute(action: string, context: ActionContext): Promise<void> {
    const handler = this.handlers.get(action);
    if (!handler) {
      console.warn(`Unknown action: ${action}`);
      context.setButtonState(context.button, "error");
      setTimeout(() => context.setButtonState(context.button, "default"), 1000);
      return;
    }

    console.log(`🔄 Executing action: ${action}`);

    // Prevent multiple clicks
    if (context.button.classList.contains("loading")) {
      return;
    }

    // Set loading state
    context.setButtonState(context.button, "loading");

    try {
      await handler(context);
    } catch (error) {
      console.error(`❌ Action failed: ${action}`, error);
      context.setButtonState(context.button, "error");
      setTimeout(() => context.setButtonState(context.button, "default"), 2000);
    }
  }

  /**
   * Check if handler exists
   */
  has(action: string): boolean {
    return this.handlers.has(action);
  }

  /**
   * Register default action handlers
   */
  private registerDefaultHandlers(): void {
    // Refresh Preview
    this.register("refresh-preview", async (context) => {
      if (context.previewManager) {
        context.previewManager.refresh();
        context.setButtonState(context.button, "success");
        setTimeout(() => context.setButtonState(context.button, "default"), 1000);
      } else {
        // Fallback: refresh iframe directly
        const iframe = document.getElementById("moss-preview-iframe") as HTMLIFrameElement;
        if (iframe && iframe.src !== "about:blank") {
          iframe.src = iframe.src; // Reload iframe
          context.setButtonState(context.button, "success");
          setTimeout(() => context.setButtonState(context.button, "default"), 1000);
        }
      }
    });

    // Setup Git Repository
    this.register("setup-git", async (context) => {
      console.log("🔧 Setting up Git...");

      try {
        const result = await TauriCommands.setupGithubRepository("", "", false, "");
        console.log("✅ Git setup result:", result);
        context.setButtonState(context.button, "success");
        setTimeout(() => context.setButtonState(context.button, "default"), 2000);
      } catch (error) {
        console.error("❌ Git setup failed:", error);
        throw error;
      }
    });

    // Show System Information
    this.register("show-system-info", async (context) => {
      try {
        const systemInfo = await TauriCommands.getSystemStatus();
        console.log("ℹ️ System Info:", systemInfo);

        // Show system info in console or could show in a modal
        alert(`System Info:\nOS: ${systemInfo.os}\nApp Version: ${systemInfo.app_version}\nFinder Integration: ${systemInfo.finder_integration ? 'Installed' : 'Not Installed'}`);

        context.setButtonState(context.button, "success");
        setTimeout(() => context.setButtonState(context.button, "default"), 1000);
      } catch (error) {
        console.error("❌ System info failed:", error);
        throw error;
      }
    });

    // Install Finder Integration
    this.register("install-finder-integration", async (context) => {
      try {
        const result = await TauriCommands.installFinderIntegration();
        console.log("✅ Finder integration result:", result);

        // Show success message
        alert(result);

        context.setButtonState(context.button, "success");
        setTimeout(() => context.setButtonState(context.button, "default"), 2000);
      } catch (error) {
        console.error("❌ Finder integration failed:", error);
        throw error;
      }
    });

    // Compile Folder (placeholder - would need folder picker)
    this.register("compile-folder", async (context) => {
      console.log("📁 Compile folder action - would show folder picker");

      // This would typically open a folder picker dialog
      // For now, just show success to demonstrate the action system
      await this.simulateAsync(1000);

      context.setButtonState(context.button, "success");
      setTimeout(() => context.setButtonState(context.button, "default"), 1000);
    });

    console.log(`✅ Registered ${this.handlers.size} default action handlers`);
  }

  /**
   * Utility function for simulating async operations
   */
  private simulateAsync(ms: number): Promise<void> {
    return new Promise((resolve) => setTimeout(resolve, ms));
  }
}