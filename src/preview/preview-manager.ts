// Preview System - Preview Window and Iframe Management
// Handles preview window lifecycle, progress display, and compilation workflow

import { type ProgressUpdate } from "../bindings";
import {
  compileWithProgress,
  getCompileConfig,
} from "../backend/tauri-integration";
import { progressTemplates } from "./progress-templates";
import { listen } from "@tauri-apps/api/event";

// FileChangeEvent type definition (mirrors Rust FileChangeEvent)
interface FileChangeEvent {
  /// Files that were deleted (source paths)
  deleted_paths?: string[] | null;
  /// Files that were renamed: (old_path, new_path) pairs
  renamed_paths?: Array<[string, string]> | null;
}

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
  private panelManager: any = null; // Will be injected later

  constructor(iframeId: string = "moss-preview-iframe") {
    this.iframe = document.getElementById(iframeId) as HTMLIFrameElement;
    if (!this.iframe) {
      throw new Error(`Preview iframe with id '${iframeId}' not found`);
    }
  }

  /**
   * Set panel manager reference for mobile/desktop mode detection
   */
  setPanelManager(panelManager: any): void {
    this.panelManager = panelManager;
  }

  /**
   * Initialize preview system
   * Check for compile config and start compilation if available
   */
  async initialize(): Promise<void> {
    console.log("🖥️ Initializing preview system...");

    // Setup file change event listener for live development
    this.setupFileChangeListener();

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

      // Start compilation with progress updates and file watching enabled
      console.log("🔍 Starting compilation with file watching enabled");
      await compileWithProgress(folderPath, autoServe, true, (update) => {
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

  /**
   * Setup event listener for file change events from backend
   */
  private async setupFileChangeListener(): Promise<void> {
    try {
      await listen<FileChangeEvent>("file-changed", (event) => {
        console.log("📁 File change event received:", event);

        // Add defensive check for payload
        if (!event || typeof event.payload === 'undefined') {
          console.warn("⚠️ Received file change event with no payload, defaulting to simple refresh");
          this.refresh();
          return;
        }

        this.handleFileChangeEvent(event.payload);
      });
      console.log("👂 File change listener setup complete");
    } catch (error) {
      console.error("❌ Failed to setup file change listener:", error);
    }
  }

  /**
   * Handle file change events during live development
   */
  private handleFileChangeEvent(changeEvent: FileChangeEvent | null): void {
    console.log("🔍 Processing file change event:", changeEvent);

    // Add null check for the entire changeEvent
    if (!changeEvent) {
      console.log("📄 Null change event, defaulting to simple refresh");
      this.refresh();
      return;
    }

    // Check if we have any changes to handle
    const hasDeleted = changeEvent.deleted_paths && changeEvent.deleted_paths.length > 0;
    const hasRenamed = changeEvent.renamed_paths && changeEvent.renamed_paths.length > 0;

    if (!hasDeleted && !hasRenamed) {
      // No special handling needed, just refresh the preview
      console.log("📄 Files modified, refreshing preview");
      this.refresh();
      return;
    }

    // Handle deletions - check if current page was deleted
    if (hasDeleted) {
      const currentUrl = this.iframe.src;
      const currentPath = this.extractPathFromUrl(currentUrl);

      if (currentPath && changeEvent.deleted_paths && changeEvent.deleted_paths.includes(currentPath)) {
        console.log("🗑️ Current page was deleted, redirecting to homepage");
        this.redirectToHomepage();
        return;
      }
    }

    // Handle renames - check if current page was renamed
    if (hasRenamed) {
      const currentUrl = this.iframe.src;
      const currentPath = this.extractPathFromUrl(currentUrl);

      if (currentPath && changeEvent.renamed_paths) {
        for (const renamePair of changeEvent.renamed_paths) {
          if (Array.isArray(renamePair) && renamePair.length >= 2) {
            const [oldPath, newPath] = renamePair;
            if (oldPath === currentPath) {
              console.log(`🔄 Current page renamed from ${oldPath} to ${newPath}, updating URL`);
              this.updateUrlForRename(oldPath, newPath);
              return;
            }
          }
        }
      }
    }

    // If we get here, files changed but current page wasn't affected
    console.log("📄 Files changed, refreshing preview");
    this.refresh();
  }

  /**
   * Extract relative path from preview URL
   */
  private extractPathFromUrl(url: string): string | null {
    try {
      const urlObj = new URL(url);
      let pathname = urlObj.pathname;

      // Remove leading slash and convert index.html to empty path
      if (pathname.startsWith("/")) {
        pathname = pathname.substring(1);
      }

      if (pathname === "index.html" || pathname === "") {
        return "index.html";
      }

      return pathname;
    } catch (error) {
      console.warn("⚠️ Failed to extract path from URL:", url, error);
      return null;
    }
  }

  /**
   * Redirect to homepage when current page is deleted
   */
  private redirectToHomepage(): void {
    try {
      const currentUrl = new URL(this.iframe.src);
      const homepageUrl = `${currentUrl.origin}/index.html`;
      console.log("🏠 Redirecting to homepage:", homepageUrl);
      this.loadPreview(homepageUrl);
    } catch (error) {
      console.error("❌ Failed to redirect to homepage:", error);
      // Fallback: just refresh the current preview
      this.refresh();
    }
  }

  /**
   * Update URL when current page is renamed
   */
  private updateUrlForRename(oldPath: string, newPath: string): void {
    try {
      const currentUrl = new URL(this.iframe.src);

      // Convert new path to URL
      let newPathWithHtml = newPath;
      if (!newPath.endsWith('.html')) {
        newPathWithHtml = newPath.replace(/\.[^/.]+$/, '') + '.html';
      }

      const newUrl = `${currentUrl.origin}/${newPathWithHtml}`;
      console.log(`🔄 Updating URL from ${oldPath} to ${newPath} (${newUrl})`);
      this.loadPreview(newUrl);
    } catch (error) {
      console.error("❌ Failed to update URL for rename:", error);
      // Fallback: refresh to let the server handle it
      this.refresh();
    }
  }

}
