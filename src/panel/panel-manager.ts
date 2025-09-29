// Panel Manager - Simplified panel system for device toggle
// Handles desktop/mobile preview mode switching

export class PanelManager {
  private panel: HTMLElement;
  private toggleButton: HTMLElement;
  private previewContainer: HTMLElement;
  private isMobileMode: boolean = false;
  private previewManager: any = null; // Will be injected later

  constructor(
    panelId: string = "moss-panel",
    triggerId: string = ".moss-panel-trigger",
    paletteId: string = "command-palette",
    backdropId: string = "command-backdrop"
  ) {
    // Get DOM elements
    this.panel = document.getElementById(panelId);
    this.previewContainer = document.querySelector(".moss-preview-container");

    if (!this.panel) throw new Error(`Panel element '${panelId}' not found`);
    if (!this.previewContainer) throw new Error(`Preview container not found`);

    // Get the device toggle button
    this.toggleButton = this.panel.querySelector(".moss-device-toggle");
    if (!this.toggleButton) throw new Error(`Device toggle button not found`);

    // Load saved mode preference
    this.loadModePreference();

    // Setup event listeners
    this.setupEventListeners();

    console.log("✅ Panel system initialized");
  }

  // Panel Management (minimal implementation for compatibility)
  showPanel(): void {
    this.panel.classList.remove("hidden");
    console.log("👁️ Panel shown");
  }

  hidePanel(): void {
    this.panel.classList.add("hidden");
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

  /**
   * Set preview manager reference for touch simulation updates
   */
  setPreviewManager(previewManager: any): void {
    this.previewManager = previewManager;
  }

  // Empty implementations for compatibility with existing code
  showCommandPalette(): void {
    console.log("Command palette functionality removed");
  }

  hideCommandPalette(): void {
    console.log("Command palette functionality removed");
  }

  toggleCommandPalette(): void {
    console.log("Command palette functionality removed");
  }

  isCommandPaletteVisible(): boolean {
    return false;
  }

  executeCommand(action: string): void {
    if (action === "toggle-preview-mode") {
      this.togglePreviewMode();
    } else {
      console.warn(`Unknown command: ${action}`);
    }
  }

  // Device Mode Management
  private setupEventListeners(): void {
    // Device toggle button
    this.toggleButton.addEventListener("click", () => {
      this.togglePreviewMode();
    });
  }

  togglePreviewMode(): void {
    this.isMobileMode = !this.isMobileMode;
    this.updatePreviewMode();
    this.saveModePreference();

    console.log(`📱 Preview mode: ${this.isMobileMode ? 'mobile' : 'desktop'}`);
  }

  private updatePreviewMode(): void {
    // Update preview container class
    if (this.isMobileMode) {
      this.previewContainer.classList.add("mobile-mode");
    } else {
      this.previewContainer.classList.remove("mobile-mode");
    }

    // Update toggle button appearance
    this.updateToggleButton();

  }

  private updateToggleButton(): void {
    const desktopIcon = this.toggleButton.querySelector(".desktop-icon") as HTMLElement;
    const mobileIcon = this.toggleButton.querySelector(".mobile-icon") as HTMLElement;

    if (this.isMobileMode) {
      // In mobile mode, show desktop icon as call to action
      if (desktopIcon) desktopIcon.style.display = "block";
      if (mobileIcon) mobileIcon.style.display = "none";
      this.toggleButton.classList.add("mobile-mode");
    } else {
      // In desktop mode, show mobile icon as call to action
      if (desktopIcon) desktopIcon.style.display = "none";
      if (mobileIcon) mobileIcon.style.display = "block";
      this.toggleButton.classList.remove("mobile-mode");
    }
  }

  private saveModePreference(): void {
    localStorage.setItem("moss-preview-mode", this.isMobileMode ? "mobile" : "desktop");
  }

  private loadModePreference(): void {
    const savedMode = localStorage.getItem("moss-preview-mode");
    this.isMobileMode = savedMode === "mobile";
    this.updatePreviewMode();
  }

  // Public API for compatibility
  getActionHandlers(): any {
    return {
      execute: (action: string) => this.executeCommand(action)
    };
  }

  getCommandRegistry(): any {
    return {
      getAll: () => []
    };
  }

  getKeyboardManager(): any {
    return {
      // Empty keyboard manager for compatibility
    };
  }

  // Get current preview mode
  isMobilePreview(): boolean {
    return this.isMobileMode;
  }

  setMobileMode(mobile: boolean): void {
    if (this.isMobileMode !== mobile) {
      this.togglePreviewMode();
    }
  }
}