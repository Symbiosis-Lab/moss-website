// Backend Integration - Tauri Commands and Progress Handling
// Provides type-safe wrappers for Tauri backend communication

import { commands, type ProgressUpdate } from "../bindings";
import { Channel } from "@tauri-apps/api/core";

// Global type declarations for Tauri integration
declare global {
  interface Window {
    tauriInvoke: typeof commands;
    __MOSS_COMPILE_CONFIG__?: {
      folder_path: string;
      auto_serve: boolean;
    };
  }
}

/**
 * Initialize Tauri backend integration
 * Makes commands available globally for other modules
 */
export async function initializeTauriBackend(): Promise<void> {
  window.tauriInvoke = commands;
  console.log("🔗 Tauri backend integration initialized");
}

/**
 * Type-safe wrapper for compilation with progress updates
 */
export async function compileWithProgress(
  folderPath: string,
  autoServe: boolean = true,
  watchOrCallback: boolean | ((update: ProgressUpdate) => void) = false,
  onProgress?: (update: ProgressUpdate) => void
): Promise<string> {
  console.log("📁 Starting compilation with progress for:", folderPath);

  // Handle backward compatibility: support both old and new calling patterns
  let watch: boolean;
  let progressCallback: (update: ProgressUpdate) => void;

  if (typeof watchOrCallback === 'function') {
    // Old calling pattern: compileWithProgress(folderPath, autoServe, onProgress)
    watch = false;
    progressCallback = watchOrCallback;
  } else {
    // New calling pattern: compileWithProgress(folderPath, autoServe, watch, onProgress)
    watch = watchOrCallback;
    progressCallback = onProgress!;
  }

  console.log("🔍 Final parameters - watch:", watch, "folderPath:", folderPath, "autoServe:", autoServe);

  // Create progress channel
  const progressChannel = new Channel<ProgressUpdate>();

  // Set up progress listener
  progressChannel.onmessage = progressCallback;

  try {
    const result = await window.tauriInvoke.compileFolder(
      folderPath,
      autoServe,
      watch,
      progressChannel
    );

    if (result.status === "ok") {
      console.log("✅ Compilation completed:", result.data);
      return result.data;
    } else {
      throw new Error(result.error);
    }
  } catch (error) {
    console.error("❌ Compilation failed:", error);
    throw error;
  }
}

/**
 * Check if compile configuration is injected
 */
export function getCompileConfig(): { folder_path: string; auto_serve: boolean } | null {
  return window.__MOSS_COMPILE_CONFIG__ || null;
}

/**
 * Type-safe wrapper for other Tauri commands
 */
export const TauriCommands = {

  async getSystemStatus() {
    try {
      const result = await window.tauriInvoke.getSystemStatus();
      if (result.status === "ok") {
        return result.data;
      } else {
        throw new Error(result.error);
      }
    } catch (error) {
      console.error("❌ System status check failed:", error);
      throw error;
    }
  },

  async installFinderIntegration() {
    try {
      const result = await window.tauriInvoke.installFinderIntegration();
      if (result.status === "ok") {
        return result.data;
      } else {
        throw new Error(result.error);
      }
    } catch (error) {
      console.error("❌ Finder integration installation failed:", error);
      throw error;
    }
  }
};