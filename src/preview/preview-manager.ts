// Preview System - Preview Window and Iframe Management
// Handles preview window lifecycle, progress display, and compilation workflow

import { type ProgressUpdate } from "../bindings";
import {
  compileWithProgress,
  getCompileConfig,
} from "../backend/tauri-integration";
import { progressTemplates } from "./progress-templates";

export interface IframeController {
  loadPreview(url: string): void;
  showMessage(message: string, isLoading?: boolean, details?: any): void;
  showProgress(message: string, percentage: number, isActive: boolean): void;
}

/**
 * Main preview system manager
 */
export class PreviewManager {
  private iframe: HTMLIFrameElement;

  constructor(iframeId: string = "moss-preview-iframe") {
    this.iframe = document.getElementById(iframeId) as HTMLIFrameElement;
    if (!this.iframe) {
      throw new Error(`Preview iframe with id '${iframeId}' not found`);
    }
  }

  /**
   * Initialize preview system
   * Check for compile config and start compilation if available
   */
  async initialize(): Promise<void> {
    console.log("🖥️ Initializing preview system...");

    const config = getCompileConfig();
    if (config) {
      console.log("🔧 Found compile config, starting compilation:", config);
      await this.startCompilation(config.folder_path, config.auto_serve);
    } else {
      this.showMessage("Welcome to moss! Right-click on a folder to publish.");
    }
  }

  /**
   * Start compilation with progress updates
   */
  async startCompilation(
    folderPath: string,
    autoServe: boolean = true
  ): Promise<void> {
    console.log("📁 Starting compilation for:", folderPath);

    try {
      // Show initial loading state
      this.showProgress("Preparing to compile...", 0, true);

      // Start compilation with progress updates
      await compileWithProgress(folderPath, autoServe, (update) => {
        this.handleProgressUpdate(update);
      });
    } catch (error) {
      console.error("❌ Compilation failed:", error);
      this.showMessage(`Failed to compile: ${error}`);
    }
  }

  /**
   * Handle progress updates from compilation
   */
  private handleProgressUpdate(update: ProgressUpdate): void {
    console.log("📊 Progress update:", update);

    // Show current progress
    this.showProgress(update.message, update.percentage, !update.completed);

    // If compilation is complete and we have a preview URL, load it
    if (update.completed && update.percentage >= 100) {
      const previewUrl = this.getPreviewUrl(update);
      if (previewUrl) {
        console.log("🌐 Loading preview:", previewUrl);
        // Brief delay to show completion, then load preview
        setTimeout(() => {
          this.loadPreview(previewUrl);
        }, 1000);
      }
    }
  }

  /**
   * Extract preview URL from progress update
   */
  private getPreviewUrl(update: ProgressUpdate): string | null {
    if (update.port) {
      return `http://localhost:${update.port}`;
    }
    return null;
  }

  /**
   * Load preview URL in iframe
   */
  loadPreview(url: string): void {
    console.log("📺 Loading preview:", url);

    this.iframe.src = url;
    // remove progress bar via srcdoc
    this.iframe.removeAttribute("srcdoc");

    // Handle iframe load errors
    this.iframe.onerror = () => {
      console.error("❌ Preview failed to load");
      this.showMessage("Failed to load preview");
    };

    console.log("📱 Preview loaded in iframe");
  }

  /**
   * Show a message in the preview area
   */
  showMessage(
    message: string,
    isLoading: boolean = false,
    details: any = null
  ): void {
    let template: string;

    if (details && details.steps) {
      template = progressTemplates.withSteps(message, details.steps);
    } else {
      template = progressTemplates.welcome(message);
    }

    const blob = new Blob([template], { type: "text/html" });
    const url = URL.createObjectURL(blob);
    this.iframe.src = url;
  }

  /**
   * Show progress state
   */
  showProgress(message: string, percentage: number, isActive: boolean): void {
    const template = isActive
      ? progressTemplates.loading(message, percentage)
      : progressTemplates.completed(message, percentage);

    this.iframe.srcdoc = template;
  }

  /**
   * Refresh current preview
   */
  refresh(): void {
    if (this.iframe && this.iframe.src !== "about:blank") {
      this.iframe.src = this.iframe.src; // Reload iframe
      console.log("🔄 Preview refreshed");
    }
  }
}
