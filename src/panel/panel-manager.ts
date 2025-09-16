// Panel Manager - Main panel system controller
// Coordinates panel visibility, command palette, and action execution

import { CommandRegistry, type Command } from "./command-registry";
import { ActionHandlers, type ButtonState, type ActionContext } from "./action-handlers";
import { KeyboardShortcutManager, type KeyboardShortcutHandler } from "./keyboard-shortcuts";

export class PanelManager implements KeyboardShortcutHandler {
  private panel: HTMLElement;
  private trigger: Element;
  private commandPalette: HTMLElement;
  private commandBackdrop: HTMLElement;
  private hideTimer: ReturnType<typeof setTimeout> | null = null;
  private isHiding: boolean = false;

  private commandRegistry: CommandRegistry;
  private actionHandlers: ActionHandlers;
  private keyboardManager: KeyboardShortcutManager;

  constructor(
    panelId: string = "moss-panel",
    triggerId: string = ".moss-panel-trigger",
    paletteId: string = "command-palette",
    backdropId: string = "command-backdrop"
  ) {
    // Get DOM elements
    this.panel = document.getElementById(panelId);
    this.trigger = document.querySelector(triggerId);
    this.commandPalette = document.getElementById(paletteId);
    this.commandBackdrop = document.getElementById(backdropId);

    if (!this.panel) throw new Error(`Panel element '${panelId}' not found`);
    if (!this.trigger) throw new Error(`Trigger element '${triggerId}' not found`);
    if (!this.commandPalette) throw new Error(`Command palette '${paletteId}' not found`);
    if (!this.commandBackdrop) throw new Error(`Command backdrop '${backdropId}' not found`);

    // Initialize subsystems
    this.commandRegistry = new CommandRegistry();
    this.actionHandlers = new ActionHandlers();
    this.keyboardManager = new KeyboardShortcutManager(this);

    // Setup event listeners
    this.setupEventListeners();
    this.setupSectionCollapsing();

    console.log("✅ Panel system initialized");
  }

  // Keyboard Shortcut Handler Interface Implementation
  showPanel(): void {
    if (this.isHiding) return;

    this.clearHideTimer();
    this.panel.classList.remove("hidden");
    console.log("👁️ Panel shown");
  }

  hidePanel(): void {
    this.panel.classList.add("hidden");
    this.clearHideTimer();
    console.log("👁️ Panel hidden");
  }

  togglePanel(): void {
    if (this.isPanelVisible()) {
      this.hidePanel();
    } else {
      this.showPanel();
    }
  }

  isPanelVisible(): boolean {
    return !this.panel.classList.contains("hidden");
  }

  showCommandPalette(): void {
    this.renderCommandPalette();
    this.commandBackdrop.classList.add("visible");
    this.commandPalette.classList.add("visible");

    // Focus search input if present
    const searchInput = this.commandPalette.querySelector("input");
    if (searchInput) {
      setTimeout(() => (searchInput as HTMLInputElement).focus(), 100);
    }

    console.log("🔍 Command palette shown");
  }

  hideCommandPalette(): void {
    this.commandBackdrop.classList.remove("visible");
    this.commandPalette.classList.remove("visible");
    console.log("🔍 Command palette hidden");
  }

  toggleCommandPalette(): void {
    if (this.isCommandPaletteVisible()) {
      this.hideCommandPalette();
    } else {
      this.showCommandPalette();
    }
  }

  isCommandPaletteVisible(): boolean {
    return this.commandPalette.classList.contains("visible");
  }

  executeCommand(action: string): void {
    const buttons = this.panel.querySelectorAll(`[data-action="${action}"]`);
    if (buttons.length > 0) {
      this.handleActionClick(buttons[0] as HTMLElement);
    } else {
      console.warn(`No button found for action: ${action}`);
    }
  }

  // Panel Management
  private setupEventListeners(): void {
    // Panel trigger hover
    this.trigger?.addEventListener("mouseenter", () => this.showPanel());

    // Panel area hover
    this.panel?.addEventListener("mouseenter", () => this.clearHideTimer());
    this.panel?.addEventListener("mouseleave", () => this.scheduleHide());

    // Panel toggle button
    const toggleBtn = this.panel.querySelector(".moss-panel-toggle");
    if (toggleBtn) {
      toggleBtn.addEventListener("click", () => this.hidePanel());
    }

    // Action buttons
    this.panel.addEventListener("click", (e) => {
      const actionBtn = (e.target as HTMLElement).closest(".moss-action-button");
      if (actionBtn) {
        e.preventDefault();
        this.handleActionClick(actionBtn as HTMLElement);
      }
    });

    // Command palette backdrop
    this.commandBackdrop.addEventListener("click", () => this.hideCommandPalette());
  }

  private scheduleHide(): void {
    this.clearHideTimer();
    this.hideTimer = setTimeout(() => {
      this.isHiding = true;
      this.hidePanel();
      setTimeout(() => {
        this.isHiding = false;
      }, 300);
    }, 3000); // 3 second delay
  }

  private clearHideTimer(): void {
    if (this.hideTimer) {
      clearTimeout(this.hideTimer);
      this.hideTimer = null;
    }
  }

  // Section Collapsing
  private setupSectionCollapsing(): void {
    const headers = this.panel.querySelectorAll(".moss-section-header");
    headers.forEach((header) => {
      header.addEventListener("click", () => {
        const isCollapsed = header.classList.toggle("collapsed");

        // Save collapse state in localStorage
        const section = (header as HTMLElement).dataset.section;
        if (section) {
          localStorage.setItem(
            `moss-section-${section}`,
            isCollapsed ? "collapsed" : "expanded"
          );
        }

        console.log(`📋 Section ${section} ${isCollapsed ? "collapsed" : "expanded"}`);
      });

      // Restore collapse state
      const section = (header as HTMLElement).dataset.section;
      if (section) {
        const savedState = localStorage.getItem(`moss-section-${section}`);
        if (savedState === "collapsed") {
          header.classList.add("collapsed");
        }
      }
    });
  }

  // Command Palette
  private renderCommandPalette(): void {
    const commands = this.commandRegistry.getAll();

    this.commandPalette.innerHTML = `
      <div style="padding: 16px;">
        <input type="text" placeholder="Type a command..."
               style="width: 100%; padding: 8px 12px; border: 1px solid var(--moss-border-medium); border-radius: 6px; font-size: 14px;"
               id="command-search">
        <div style="margin-top: 12px; max-height: 300px; overflow-y: auto;">
          ${commands
            .map(
              (cmd) => `
            <div class="command-item" data-action="${cmd.action}"
                 style="padding: 8px 12px; border-radius: 4px; cursor: pointer; display: flex; align-items: center; gap: 12px;"
                 onmouseover="this.style.background='var(--moss-background-alt)'"
                 onmouseout="this.style.background='transparent'">
              <div style="width: 24px; height: 24px;">${cmd.icon || "⚙️"}</div>
              <div>
                <div style="font-weight: 500;">${cmd.name}</div>
                <div style="font-size: 12px; color: var(--moss-text-secondary);">${cmd.section}</div>
              </div>
            </div>
          `
            )
            .join("")}
        </div>
      </div>
    `;

    this.setupCommandPaletteInteractions();
  }

  private setupCommandPaletteInteractions(): void {
    const searchInput = this.commandPalette.querySelector("#command-search");
    const commandItems = this.commandPalette.querySelectorAll(".command-item");

    // Search functionality
    if (searchInput) {
      searchInput.addEventListener("input", (e) => {
        const query = (e.target as HTMLInputElement).value.toLowerCase();
        commandItems.forEach((item) => {
          const text = item.textContent!.toLowerCase();
          (item as HTMLElement).style.display = text.includes(query) ? "flex" : "none";
        });
      });

      searchInput.addEventListener("keydown", (e: KeyboardEvent) => {
        if (e.key === "Enter") {
          const visibleItems = Array.from(commandItems).filter(
            (item) => (item as HTMLElement).style.display !== "none"
          );
          if (visibleItems.length > 0) {
            (visibleItems[0] as HTMLElement).click();
          }
        }
      });
    }

    // Click handlers
    commandItems.forEach((item) => {
      item.addEventListener("click", () => {
        const action = (item as HTMLElement).dataset.action;
        this.executeCommand(action!);
        this.hideCommandPalette();
      });
    });
  }

  // Action Handling
  private async handleActionClick(button: HTMLElement): Promise<void> {
    const action = button.dataset.action;
    if (!action) return;

    const context: ActionContext = {
      button,
      setButtonState: this.setButtonState.bind(this),
      previewManager: undefined, // Can be injected if needed
    };

    await this.actionHandlers.execute(action, context);
  }

  private setButtonState(button: HTMLElement, state: ButtonState): void {
    // Remove all state classes
    button.classList.remove("loading", "success", "error", "active");

    // Add new state class
    if (state !== "default") {
      button.classList.add(state);
    }
  }

  // Public API
  getCommandRegistry(): CommandRegistry {
    return this.commandRegistry;
  }

  getActionHandlers(): ActionHandlers {
    return this.actionHandlers;
  }

  getKeyboardManager(): KeyboardShortcutManager {
    return this.keyboardManager;
  }
}