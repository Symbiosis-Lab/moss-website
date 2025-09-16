// Keyboard Shortcuts - Global keyboard event handling
// Manages keyboard shortcuts for panel and command palette operations

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

/**
 * Keyboard shortcut manager
 */
export class KeyboardShortcutManager {
  private handler: KeyboardShortcutHandler;
  private isEnabled: boolean = true;

  constructor(handler: KeyboardShortcutHandler) {
    this.handler = handler;
    this.setupKeyboardListeners();
    console.log("⌨️ Keyboard shortcuts initialized");
  }

  /**
   * Enable/disable keyboard shortcuts
   */
  setEnabled(enabled: boolean): void {
    this.isEnabled = enabled;
    console.log(`⌨️ Keyboard shortcuts ${enabled ? 'enabled' : 'disabled'}`);
  }

  /**
   * Setup global keyboard event listeners
   */
  private setupKeyboardListeners(): void {
    document.addEventListener("keydown", (e) => {
      if (!this.isEnabled) return;

      // Command palette: Cmd+Shift+P (Mac) or Ctrl+Shift+P (PC)
      if (e.key === "P" && e.shiftKey && (e.metaKey || e.ctrlKey)) {
        e.preventDefault();
        this.handler.toggleCommandPalette();
        return;
      }

      // Panel toggle: Cmd+/ (Mac) or Ctrl+/ (PC)
      if (e.key === "/" && (e.metaKey || e.ctrlKey)) {
        e.preventDefault();
        this.handler.togglePanel();
        return;
      }

      // Refresh preview: Cmd+R (Mac) or Ctrl+R (PC)
      if (e.key === "r" && (e.metaKey || e.ctrlKey)) {
        e.preventDefault();
        this.handler.executeCommand("refresh-preview");
        return;
      }

      // Compile folder: Cmd+B (Mac) or Ctrl+B (PC)
      if (e.key === "b" && (e.metaKey || e.ctrlKey)) {
        e.preventDefault();
        this.handler.executeCommand("compile-folder");
        return;
      }

      // Show system info: Cmd+I (Mac) or Ctrl+I (PC)
      if (e.key === "i" && (e.metaKey || e.ctrlKey)) {
        e.preventDefault();
        this.handler.executeCommand("show-system-info");
        return;
      }

      // Hide panel/palette: Escape
      if (e.key === "Escape") {
        if (this.handler.isCommandPaletteVisible()) {
          this.handler.hideCommandPalette();
        } else if (this.handler.isPanelVisible()) {
          this.handler.hidePanel();
        }
        return;
      }
    });
  }

  /**
   * Get list of available shortcuts for help/documentation
   */
  getShortcuts(): Array<{ key: string; description: string; action?: string }> {
    return [
      {
        key: "Cmd/Ctrl + Shift + P",
        description: "Toggle command palette",
      },
      {
        key: "Cmd/Ctrl + /",
        description: "Toggle panel",
      },
      {
        key: "Cmd/Ctrl + R",
        description: "Refresh preview",
        action: "refresh-preview",
      },
      {
        key: "Cmd/Ctrl + B",
        description: "Compile folder",
        action: "compile-folder",
      },
      {
        key: "Cmd/Ctrl + I",
        description: "Show system information",
        action: "show-system-info",
      },
      {
        key: "Escape",
        description: "Close panel or command palette",
      },
    ];
  }

  /**
   * Show help message with shortcuts
   */
  showHelp(): void {
    const shortcuts = this.getShortcuts();
    const helpText = shortcuts
      .map(shortcut => `${shortcut.key}: ${shortcut.description}`)
      .join('\n');

    console.log("⌨️ Available keyboard shortcuts:\n" + helpText);

    // Could also show in a modal or help panel
    alert("Keyboard Shortcuts:\n\n" + helpText);
  }
}