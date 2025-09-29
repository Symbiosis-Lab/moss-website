// App & UI Module - Application initialization and UI utilities
// Combines application coordination with UI management

import { initializeTauriBackend } from "./backend/tauri-integration";
import { PreviewManager } from "./preview/preview-manager";
import { PanelManager } from "./panel/panel-manager";

// Global type declarations
declare global {
  interface Window {
    mossPanelSystem: PanelManager;
    mossPreviewManager: PreviewManager;
  }
}

/**
 * Main application class that coordinates all subsystems
 */
export class MossApp {
  private previewManager: PreviewManager | null = null;
  private panelManager: PanelManager | null = null;
  private isInitialized: boolean = false;

  /**
   * Initialize the complete moss application
   */
  async initialize(): Promise<void> {
    if (this.isInitialized) {
      console.warn("⚠️ App already initialized");
      return;
    }

    console.log("🌿 Initializing moss Tauri app...");

    try {
      // Initialize backend first (required for other systems)
      await initializeTauriBackend();

      // Initialize preview system
      await this.initializePreviewSystem();

      // Initialize panel system
      this.initializePanelSystem();

      // Store references globally for external access
      if (this.panelManager) {
        window.mossPanelSystem = this.panelManager;
      }
      if (this.previewManager) {
        window.mossPreviewManager = this.previewManager;
      }

      this.isInitialized = true;
      console.log("✅ moss app initialization complete");
    } catch (error) {
      console.error("❌ App initialization failed:", error);
      throw error;
    }
  }

  /**
   * Initialize preview system
   */
  private async initializePreviewSystem(): Promise<void> {
    console.log("🖥️ Initializing preview system...");

    try {
      this.previewManager = new PreviewManager("moss-preview-iframe");
      await this.previewManager.initialize();
      console.log("✅ Preview system initialized");
    } catch (error) {
      console.error("❌ Preview system initialization failed:", error);
      throw error;
    }
  }

  /**
   * Initialize panel system
   */
  private initializePanelSystem(): void {
    console.log("🎨 Initializing panel system...");

    try {
      this.panelManager = new PanelManager(
        "moss-panel",
        ".moss-panel-trigger",
        "command-palette",
        "command-backdrop"
      );

      // Connect preview manager to panel manager for mode detection
      if (this.previewManager && this.panelManager) {
        this.previewManager.setPanelManager(this.panelManager);
        this.panelManager.setPreviewManager(this.previewManager);
      }

      console.log("✅ Panel system initialized");
    } catch (error) {
      console.error("❌ Panel system initialization failed:", error);
      throw error;
    }
  }

  /**
   * Get preview manager instance
   */
  getPreviewManager(): PreviewManager | null {
    return this.previewManager;
  }

  /**
   * Get panel manager instance
   */
  getPanelManager(): PanelManager | null {
    return this.panelManager;
  }

  /**
   * Check if app is initialized
   */
  isReady(): boolean {
    return this.isInitialized;
  }

  /**
   * Cleanup resources
   */
  destroy(): void {
    console.log("🧹 Cleaning up moss app...");

    // Could add cleanup logic here if needed
    // - Remove event listeners
    // - Clear timers
    // - Reset global state

    this.isInitialized = false;
    console.log("✅ App cleanup complete");
  }
}

/**
 * UI Utilities - Helper functions for common UI operations
 */
export class UIUtils {
  /**
   * Show a temporary notification message
   */
  static showNotification(message: string, type: "info" | "success" | "error" = "info", duration: number = 3000): void {
    // Create notification element
    const notification = document.createElement("div");
    notification.className = `moss-notification moss-notification-${type}`;
    notification.textContent = message;

    // Style the notification
    Object.assign(notification.style, {
      position: "fixed",
      top: "20px",
      right: "20px",
      padding: "12px 16px",
      borderRadius: "8px",
      color: "white",
      fontFamily: "-apple-system, BlinkMacSystemFont, sans-serif",
      fontSize: "14px",
      fontWeight: "500",
      zIndex: "10000",
      boxShadow: "0 4px 12px rgba(0, 0, 0, 0.15)",
      transform: "translateX(100%)",
      transition: "transform 0.3s ease",
      backgroundColor: type === "error" ? "#ef4444" : type === "success" ? "#10b981" : "#6366f1"
    });

    // Add to DOM
    document.body.appendChild(notification);

    // Animate in
    requestAnimationFrame(() => {
      notification.style.transform = "translateX(0)";
    });

    // Remove after duration
    setTimeout(() => {
      notification.style.transform = "translateX(100%)";
      setTimeout(() => {
        if (notification.parentNode) {
          notification.parentNode.removeChild(notification);
        }
      }, 300);
    }, duration);
  }

  /**
   * Create a loading spinner element
   */
  static createSpinner(size: number = 20): HTMLElement {
    const spinner = document.createElement("div");
    spinner.className = "moss-spinner";

    Object.assign(spinner.style, {
      width: `${size}px`,
      height: `${size}px`,
      border: "2px solid #e5e7eb",
      borderTop: "2px solid #6366f1",
      borderRadius: "50%",
      animation: "moss-spin 1s linear infinite",
      display: "inline-block"
    });

    // Add keyframes if not already present
    if (!document.querySelector("#moss-spinner-styles")) {
      const style = document.createElement("style");
      style.id = "moss-spinner-styles";
      style.textContent = `
        @keyframes moss-spin {
          from { transform: rotate(0deg); }
          to { transform: rotate(360deg); }
        }
      `;
      document.head.appendChild(style);
    }

    return spinner;
  }

  /**
   * Safe DOM element selection with error handling
   */
  static getElement<T extends Element>(selector: string, required: boolean = true): T | null {
    const element = document.querySelector<T>(selector);

    if (required && !element) {
      console.error(`❌ Required element not found: ${selector}`);
      throw new Error(`Element not found: ${selector}`);
    }

    return element;
  }

  /**
   * Safe DOM element by ID with error handling
   */
  static getElementById<T extends HTMLElement>(id: string, required: boolean = true): T | null {
    const element = document.getElementById(id) as T;

    if (required && !element) {
      console.error(`❌ Required element not found with ID: ${id}`);
      throw new Error(`Element not found with ID: ${id}`);
    }

    return element;
  }

  /**
   * Debounce function for event handling
   */
  static debounce<T extends (...args: any[]) => any>(
    func: T,
    wait: number
  ): (...args: Parameters<T>) => void {
    let timeout: ReturnType<typeof setTimeout>;
    return (...args: Parameters<T>) => {
      clearTimeout(timeout);
      timeout = setTimeout(() => func.apply(null, args), wait);
    };
  }

  /**
   * Throttle function for event handling
   */
  static throttle<T extends (...args: any[]) => any>(
    func: T,
    limit: number
  ): (...args: Parameters<T>) => void {
    let inThrottle: boolean;
    return (...args: Parameters<T>) => {
      if (!inThrottle) {
        func.apply(null, args);
        inThrottle = true;
        setTimeout(() => (inThrottle = false), limit);
      }
    };
  }
}

/**
 * Export singleton app instance for easy access
 */
export const mossApp = new MossApp();