// moss Tauri App - Main Frontend Entry Point
// This is the build-time frontend for the Tauri application

import { commands, type ProgressUpdate } from './bindings';
import { Channel } from "@tauri-apps/api/core";

document.addEventListener("DOMContentLoaded", async () => {
  console.log("🌿 moss Tauri app loaded");

  // Initialize components
  await initializePreviewSystem();
  await initializeTauriBackend();
  initializeRightPanelSystem();

  console.log("✅ moss app initialization complete");
});

// Type-safe global reference to invoke function
declare global {
  interface Window {
    tauriInvoke: typeof commands;
    mossPanelSystem: MossPanelSystem;
  }
}

// Tauri Backend Integration
async function initializeTauriBackend(): Promise<void> {

  // const { getCurrent, onOpenUrl } = await import(
  //   "@tauri-apps/plugin-deep-link"
  // );

  // TODO: move deeplink logic to backend
  // try {
  //   console.log("Testing backend connection...");
  //   const systemStatus = await invoke("get_system_status");
  //   console.log("✅ Backend connected:", systemStatus);
  // } catch (error) {
  //   console.error("❌ Backend connection failed:", error);
  // }

  // // Handle deep links (e.g., moss://publish?path=/path/to/folder)
  // try {
  //   const urls = await getCurrent();
  //   if (urls && urls.length > 0) {
  //     console.log("🔗 App started via deep link:", urls);
  //     for (const url of urls) {
  //       await handleDeepLink(url);
  //     }
  //   }
  // } catch (error) {
  //   console.log("No deep link on startup (this is normal)");
  // }

  // // Listen for deep links while app is running
  // try {
  //   console.log("🎯 Setting up runtime deep link listener...");
  //   await onOpenUrl(async (urls) => {
  //     console.log("🔗 Runtime deep link received:", urls);
  //     for (const url of urls) {
  //       await handleDeepLink(url);
  //     }
  //   });
  //   console.log("✅ Runtime deep link listener active");
  // } catch (error) {
  //   console.error("❌ Failed to set up runtime deep link listener:", error);
  // }

  // TODO: fix context menu setup and deep link support. Finder integration should be moved to backend
  // Check if this is first launch - install Finder integration
  // try {
  //   console.log("🔧 Checking Finder integration...");
  //   const integrationResult = await invoke("install_finder_integration");
  //   console.log("✅ Finder integration:", integrationResult);
  // } catch (error) {
  //   console.error("❌ Failed to install Finder integration:", error);
  // }

  // Store type-safe commands globally for use in panel system
  window.tauriInvoke = commands;
}

// async function handleDeepLink(url) {
//   console.log("Processing deep link:", url);

//   try {
//     const urlObj = new URL(url);

//     // Handle moss://publish?path=/path/to/folder
//     if (urlObj.protocol === "moss:" && urlObj.pathname === "//publish") {
//       const folderPath = urlObj.searchParams.get("path");
//       if (folderPath) {
//         console.log("📁 Publishing folder via deep link:", folderPath);

//         await startCompilationWithProgress(
//           decodeURIComponent(folderPath),
//           true
//         );
//       } else {
//         console.error("❌ No folder path in deep link");
//         showPreviewMessage("Error: No folder path provided");
//       }
//     } else {
//       console.log("ℹ️ Unknown deep link format:", url);
//     }
//   } catch (error) {
//     console.error("❌ Failed to parse deep link:", error);
//     showPreviewMessage("Error: Failed to process link");
//   }
// }

// Preview System with Iframe
async function initializePreviewSystem() {
  console.log("🖥️ Initializing preview system...");

  // Don't automatically check for preview server on startup
  // Only show preview when explicitly requested via IPC events
  await setupPreviewEventListeners();
  console.log("Preview system initialized, waiting for preview requests");
  showPreviewMessage("Welcome to moss! Right-click on a folder to publish.");
}

function loadPreview(url) {
  console.log("📺 Loading preview:", url);

  const iframe = document.getElementById("moss-preview-iframe");

  // Set iframe source
  iframe.src = url;

  hideProgressOverlay();

  // Handle iframe load errors
  iframe.onerror = () => {
    console.error("❌ Preview failed to load");
    showPreviewMessage("Failed to load preview");
  };
}

function showPreviewMessage(message, isLoading = false, details = null) {
  const iframe = document.getElementById("moss-preview-iframe");
  const spinnerStyle = isLoading
    ? `
    .spinner {
      border: 3px solid #e5e7eb;
      border-top: 3px solid #6366f1;
      border-radius: 50%;
      width: 24px;
      height: 24px;
      animation: spin 1s linear infinite;
      margin: 0 auto 1rem auto;
    }
    @keyframes spin {
      0% { transform: rotate(0deg); }
      100% { transform: rotate(360deg); }
    }
    .progress-steps {
      margin-top: 1.5rem;
      padding: 1rem;
      background: rgba(255, 255, 255, 0.8);
      border-radius: 8px;
      font-size: 14px;
      text-align: left;
      max-width: 300px;
    }
    .step {
      display: flex;
      align-items: center;
      margin: 0.5rem 0;
      padding: 0.25rem 0;
    }
    .step-icon {
      width: 16px;
      margin-right: 8px;
      text-align: center;
    }
    .step.completed { color: #10b981; }
    .step.active { color: #6366f1; font-weight: 500; }
    .step.pending { color: #9ca3af; }
  `
    : "";

  const spinnerHtml = isLoading ? '<div class="spinner"></div>' : "";

  // Generate progress steps if details are provided
  let progressHtml = "";
  if (details && details.steps) {
    const steps = details.steps
      .map((step, index) => {
        let stepClass = "pending";
        let icon = "⏳";

        if (step.status === "completed") {
          stepClass = "completed";
          icon = "✅";
        } else if (step.status === "active") {
          stepClass = "active";
          icon = "🔄";
        } else if (step.status === "error") {
          stepClass = "error";
          icon = "❌";
        }

        return `
        <div class="step ${stepClass}">
          <span class="step-icon">${icon}</span>
          <span>${step.name}</span>
        </div>
      `;
      })
      .join("");

    progressHtml = `
      <div class="progress-steps">
        <div style="font-weight: 600; margin-bottom: 0.75rem; color: #374151;">Progress:</div>
        ${steps}
      </div>
    `;
  }

  const htmlContent = `
    <!DOCTYPE html>
    <html>
    <head>
      <title>moss</title>
      <style>
        body { 
          font-family: -apple-system, BlinkMacSystemFont, sans-serif; 
          display: flex; 
          align-items: center; 
          justify-content: center; 
          height: 100vh; 
          margin: 0; 
          background: #f8f9fa; 
          color: #6b7280;
        }
        .message {
          text-align: center;
          padding: 2rem;
          max-width: 500px;
        }
        .moss-icon {
          font-size: 48px;
          margin-bottom: 1rem;
        }
        ${spinnerStyle}
      </style>
    </head>
    <body>
      <div class="message">
        <div class="moss-icon">🌿</div>
        ${spinnerHtml}
        <p>${message}</p>
        ${progressHtml}
      </div>
    </body>
    </html>
  `;

  const blob = new Blob([htmlContent], { type: "text/html" });
  const url = URL.createObjectURL(blob);
  iframe.src = url;
}

// Real-time progress update handler
let currentProgressSteps = {};

// Unified function to start compilation with progress channel
async function startCompilationWithProgress(folderPath: string, autoServe: boolean = true): Promise<void> {
  console.log("📁 Starting compilation with progress for:", folderPath);

  try {
    // Reset progress state
    currentProgressSteps = {};

    // Start with loading message
    showPreviewMessage("Preparing to compile...", true);

    // Use Channel API to get real-time progress
    const progressChannel = new Channel<ProgressUpdate>();

    // Listen for progress updates
    progressChannel.onmessage = (progressUpdate: ProgressUpdate) => {
      console.log("📊 Progress update:", progressUpdate);
      showProgressUpdate(progressUpdate);
    };

    const result = await window.tauriInvoke.compileFolder(folderPath, autoServe, progressChannel);
    if (result.status === "ok") {
      console.log("✅ Compile result:", result.data);
    } else {
      console.error("❌ Compilation failed:", result.error);
      showPreviewMessage("Failed to compile: " + result.error);
    }
  } catch (error) {
    console.error("❌ Compilation failed:", error);
    showPreviewMessage("Failed to compile: " + error);
  }
}

function showProgressUpdate(progressUpdate: ProgressUpdate): void {
  console.log("📊 Progress update:", progressUpdate);

  // Show only the current step with smooth transition
  showCurrentProgress(
    progressUpdate.message,
    progressUpdate.percentage,
    !progressUpdate.completed
  );

  // If this is the final step with a preview URL, load it in iframe
  if (progressUpdate.completed && progressUpdate.percentage >= 100) {
    // Check if progress update includes a preview URL or port
    let previewUrl = null;

    if (progressUpdate.url) {
      previewUrl = progressUpdate.url;
    } else if (progressUpdate.port) {
      previewUrl = `http://localhost:${progressUpdate.port}`;
    }

    if (previewUrl) {
      console.log("🌐 Loading preview in iframe:", previewUrl);

      // Brief delay to show completion, then load preview
      setTimeout(() => {
        loadPreview(previewUrl);
      }, 1000);
    } else {
      // No URL found, just hide progress after showing completion
      setTimeout(() => {
        hideProgressOverlay();
      }, 1000);
    }
  }
}

function showCurrentProgress(message, percentage, isActive) {
  const iframe = document.getElementById("moss-preview-iframe");

  // Create smooth progress display with single step
  const progressHtml = `
    <div class="progress-container">
      <style>
        .progress-container {
          display: flex;
          flex-direction: column;
          align-items: center;
          justify-content: center;
          height: 100vh;
          background: #fafafa;
          font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
          transition: all 0.3s ease;
        }

        .progress-step {
          text-align: center;
          max-width: 400px;
          padding: 32px;
          animation: fadeInUp 0.4s ease-out;
        }

        .progress-message {
          font-size: 18px;
          color: #374151;
          margin-bottom: 24px;
          font-weight: 500;
          line-height: 1.4;
        }

        .progress-bar-container {
          width: 100%;
          height: 4px;
          background: #e5e7eb;
          border-radius: 2px;
          overflow: hidden;
          margin-bottom: 16px;
        }

        .progress-bar {
          height: 100%;
          background: linear-gradient(90deg, #6366f1, #8b5cf6);
          border-radius: 2px;
          transition: width 0.6s cubic-bezier(0.4, 0, 0.2, 1);
          width: ${percentage}%;
        }

        .progress-percentage {
          font-size: 14px;
          color: #6b7280;
          font-weight: 500;
        }

        .spinner {
          width: 20px;
          height: 20px;
          border: 2px solid #e5e7eb;
          border-top: 2px solid #6366f1;
          border-radius: 50%;
          animation: spin 1s linear infinite;
          margin: 0 auto 16px;
          display: ${isActive ? "block" : "none"};
        }

        @keyframes fadeInUp {
          from {
            opacity: 0;
            transform: translateY(20px);
          }
          to {
            opacity: 1;
            transform: translateY(0);
          }
        }

        @keyframes spin {
          from { transform: rotate(0deg); }
          to { transform: rotate(360deg); }
        }

        .completion-check {
          width: 24px;
          height: 24px;
          border-radius: 50%;
          background: #10b981;
          display: ${isActive ? "none" : "flex"};
          align-items: center;
          justify-content: center;
          margin: 0 auto 16px;
          color: white;
          font-size: 14px;
          animation: scaleIn 0.3s ease-out;
        }

        @keyframes scaleIn {
          from {
            opacity: 0;
            transform: scale(0.8);
          }
          to {
            opacity: 1;
            transform: scale(1);
          }
        }
      </style>

      <div class="progress-step">
        <div class="spinner"></div>
        <div class="completion-check">✓</div>
        <div class="progress-message">${message}</div>
        <div class="progress-bar-container">
          <div class="progress-bar"></div>
        </div>
        <div class="progress-percentage">${percentage}%</div>
      </div>
    </div>
  `;

  iframe.srcdoc = progressHtml;
}

function hideProgressOverlay() {
  // Progress overlay is hidden when iframe loads actual content
  const iframe = document.getElementById("moss-preview-iframe");
  iframe.removeAttribute("srcdoc");

  console.log("📱 Progress overlay hidden - iframe content loaded");
}

// Right Panel System
function initializeRightPanelSystem() {
  console.log("🎨 Initializing right panel system...");

  const panelSystem = new MossPanelSystem();

  // Store globally for access from other functions
  window.mossPanelSystem = panelSystem;
}

interface Command {
  action: string;
  name: string;
  section: string;
  icon?: string;
}

class MossPanelSystem {
  private panel: HTMLElement | null;
  private trigger: Element | null;
  private commandPalette: HTMLElement | null;
  private commandBackdrop: HTMLElement | null;
  private hideTimer: number | null;
  private isHiding: boolean;
  private commands: Map<string, Command>;

  constructor() {
    this.panel = document.getElementById("moss-panel");
    this.trigger = document.querySelector(".moss-panel-trigger");
    this.commandPalette = document.getElementById("command-palette");
    this.commandBackdrop = document.getElementById("command-backdrop");
    this.hideTimer = null;
    this.isHiding = false;

    // Command registry
    this.commands = new Map<string, Command>();

    this.setupEventListeners();
    this.setupKeyboardShortcuts();
    this.setupSectionCollapsing();
    this.registerDefaultActions();

    console.log("✅ Panel system initialized");
  }

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
      const actionBtn = e.target.closest(".moss-action-button");
      if (actionBtn) {
        e.preventDefault();
        this.handleActionClick(actionBtn);
      }
    });

    // Command palette backdrop
    if (this.commandBackdrop) {
      this.commandBackdrop.addEventListener("click", () =>
        this.hideCommandPalette()
      );
    }
  }

  setupKeyboardShortcuts() {
    document.addEventListener("keydown", (e) => {
      // Command palette: Cmd+Shift+P (Mac) or Ctrl+Shift+P (PC)
      if (e.key === "P" && e.shiftKey && (e.metaKey || e.ctrlKey)) {
        e.preventDefault();
        this.toggleCommandPalette();
        return;
      }

      // Panel toggle: Cmd+/ (Mac) or Ctrl+/ (PC)
      if (e.key === "/" && (e.metaKey || e.ctrlKey)) {
        e.preventDefault();
        this.togglePanel();
        return;
      }

      // Refresh preview: Cmd+R (Mac) or Ctrl+R (PC)
      if (e.key === "r" && (e.metaKey || e.ctrlKey)) {
        e.preventDefault();
        this.executeCommand("refresh-preview");
        return;
      }

      // Hide panel/palette: Escape
      if (e.key === "Escape") {
        if (this.isCommandPaletteVisible()) {
          this.hideCommandPalette();
        } else if (this.isPanelVisible()) {
          this.hidePanel();
        }
        return;
      }
    });
  }

  setupSectionCollapsing() {
    const headers = this.panel.querySelectorAll(".moss-section-header");
    headers.forEach((header) => {
      header.addEventListener("click", () => {
        const isCollapsed = header.classList.toggle("collapsed");

        // Save collapse state in localStorage
        const section = header.dataset.section;
        if (section) {
          localStorage.setItem(
            `moss-section-${section}`,
            isCollapsed ? "collapsed" : "expanded"
          );
        }

        console.log(
          `📋 Section ${section} ${isCollapsed ? "collapsed" : "expanded"}`
        );
      });

      // Restore collapse state
      const section = header.dataset.section;
      if (section) {
        const savedState = localStorage.getItem(`moss-section-${section}`);
        if (savedState === "collapsed") {
          header.classList.add("collapsed");
        }
      }
    });
  }

  // Panel visibility management
  showPanel() {
    if (this.isHiding) return;

    this.clearHideTimer();
    this.panel.classList.remove("hidden");
    console.log("👁️ Panel shown");
  }

  hidePanel() {
    this.panel.classList.add("hidden");
    this.clearHideTimer();
    console.log("👁️ Panel hidden");
  }

  togglePanel() {
    if (this.isPanelVisible()) {
      this.hidePanel();
    } else {
      this.showPanel();
    }
  }

  isPanelVisible() {
    return !this.panel.classList.contains("hidden");
  }

  scheduleHide() {
    this.clearHideTimer();
    this.hideTimer = setTimeout(() => {
      this.isHiding = true;
      this.hidePanel();
      setTimeout(() => {
        this.isHiding = false;
      }, 300);
    }, 3000); // 3 second delay
  }

  clearHideTimer() {
    if (this.hideTimer) {
      clearTimeout(this.hideTimer);
      this.hideTimer = null;
    }
  }

  // Command palette management
  toggleCommandPalette() {
    if (this.isCommandPaletteVisible()) {
      this.hideCommandPalette();
    } else {
      this.showCommandPalette();
    }
  }

  showCommandPalette() {
    if (!this.commandPalette || !this.commandBackdrop) return;

    this.renderCommandPalette();
    this.commandBackdrop.classList.add("visible");
    this.commandPalette.classList.add("visible");

    // Focus search input if present
    const searchInput = this.commandPalette.querySelector("input");
    if (searchInput) {
      setTimeout(() => searchInput.focus(), 100);
    }

    console.log("🔍 Command palette shown");
  }

  hideCommandPalette() {
    if (!this.commandPalette || !this.commandBackdrop) return;

    this.commandBackdrop.classList.remove("visible");
    this.commandPalette.classList.remove("visible");
    console.log("🔍 Command palette hidden");
  }

  isCommandPaletteVisible() {
    return (
      this.commandPalette && this.commandPalette.classList.contains("visible")
    );
  }

  renderCommandPalette() {
    if (!this.commandPalette) return;

    const commands = Array.from(this.commands.values());

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
                <div style="font-size: 12px; color: var(--moss-text-secondary);">${
                  cmd.section
                }</div>
              </div>
            </div>
          `
            )
            .join("")}
        </div>
      </div>
    `;

    // Add search functionality
    const searchInput = this.commandPalette.querySelector("#command-search");
    const commandItems = this.commandPalette.querySelectorAll(".command-item");

    if (searchInput) {
      searchInput.addEventListener("input", (e) => {
        const query = e.target.value.toLowerCase();
        commandItems.forEach((item) => {
          const text = item.textContent.toLowerCase();
          item.style.display = text.includes(query) ? "flex" : "none";
        });
      });

      searchInput.addEventListener("keydown", (e) => {
        if (e.key === "Enter") {
          const visibleItems = Array.from(commandItems).filter(
            (item) => item.style.display !== "none"
          );
          if (visibleItems.length > 0) {
            visibleItems[0].click();
          }
        }
      });
    }

    // Add click handlers
    commandItems.forEach((item) => {
      item.addEventListener("click", () => {
        const action = item.dataset.action;
        this.executeCommand(action);
        this.hideCommandPalette();
      });
    });
  }

  // Action handling
  async handleActionClick(button) {
    const action = button.dataset.action;
    const command = button.dataset.command;

    console.log(`🔄 Executing action: ${action}`);

    // Prevent multiple clicks
    if (button.classList.contains("loading")) {
      return;
    }

    // Set loading state
    this.setButtonState(button, "loading");

    try {
      await this.executeAction(action, button);
    } catch (error) {
      console.error(`❌ Action failed: ${action}`, error);
      this.setButtonState(button, "error");
      setTimeout(() => this.setButtonState(button, "default"), 2000);
    }
  }

  async executeAction(action, button) {
    // Handle built-in actions
    switch (action) {
      case "refresh-preview":
        const iframe = document.getElementById("moss-preview-iframe");
        if (iframe && iframe.src !== "about:blank") {
          iframe.src = iframe.src; // Reload iframe
          this.setButtonState(button, "success");
          setTimeout(() => this.setButtonState(button, "default"), 1000);
        }
        return;

      case "setup-git":
        // This integrates with the Tauri backend
        console.log("🔧 Setting up Git...");
        if (window.tauriInvoke) {
          try {
            const result = await window.tauriInvoke("setup_git_repository", {});
            console.log("✅ Git setup result:", result);
            this.setButtonState(button, "success");
            setTimeout(() => this.setButtonState(button, "default"), 2000);
          } catch (error) {
            console.error("❌ Git setup failed:", error);
            throw error;
          }
        } else {
          // Fallback simulation for development
          await this.simulateAsync(1000);
          this.setButtonState(button, "success");
          setTimeout(() => this.setButtonState(button, "default"), 1000);
        }
        return;

      default:
        console.warn(`Unknown action: ${action}`);
        this.setButtonState(button, "error");
        setTimeout(() => this.setButtonState(button, "default"), 1000);
    }
  }

  executeCommand(action) {
    const buttons = this.panel.querySelectorAll(`[data-action="${action}"]`);
    if (buttons.length > 0) {
      this.handleActionClick(buttons[0]);
    } else {
      console.warn(`No button found for action: ${action}`);
    }
  }

  setButtonState(button, state) {
    // Remove all state classes
    button.classList.remove("loading", "success", "error", "active");

    // Add new state class
    if (state !== "default") {
      button.classList.add(state);
    }
  }

  // Register default actions and commands
  registerDefaultActions() {
    this.commands.set("refresh-preview", {
      action: "refresh-preview",
      name: "Refresh Preview",
      section: "Settings",
      icon: "🔄",
    });

    this.commands.set("setup-git", {
      action: "setup-git",
      name: "Setup Git Repository",
      section: "Publish",
      icon: "⚙️",
    });
  }

  // Utility function for simulating async operations
  simulateAsync(ms) {
    return new Promise((resolve) => setTimeout(resolve, ms));
  }
}

// Preview Event Listeners - Listen for IPC events from backend
async function setupPreviewEventListeners() {
  console.log("🎧 Setting up preview event listeners...");

  try {
    const { listen } = await import("@tauri-apps/api/event");
    console.log("✅ Successfully imported event listener");

    // Listen for compilation started events
    await listen("preview-compilation-started", (event) => {
      console.log("🔧 Compilation started:", event.payload);

      const { message, folder_path } = event.payload;

      // Show detailed progress steps
      const progressSteps = {
        steps: [
          { name: "Reading source files", status: "active" },
          { name: "Processing templates", status: "pending" },
          { name: "Generating pages", status: "pending" },
          { name: "Starting local server", status: "pending" },
        ],
      };

      showPreviewMessage(
        message || "Compiling website...",
        true,
        progressSteps
      );
    });

    // Listen for "ready for compile" events from menu bar flow
    console.log("🎧 Setting up ready-for-compile listener...");
    const unlistenReadyForCompile = await listen(
      "ready-for-compile",
      async (event) => {
        console.log(
          "🚀 RECEIVED ready-for-compile event from menu bar:",
          event.payload
        );
        console.log("🚀 Event details:", JSON.stringify(event, null, 2));

        const { folder_path, auto_serve } = event.payload;
        console.log("📁 Folder path:", folder_path, "Auto serve:", auto_serve);

        try {
          console.log("🔧 Calling startCompilationWithProgress...");
          // Frontend creates channel and calls backend compile function
          await startCompilationWithProgress(folder_path, auto_serve);
          console.log("✅ startCompilationWithProgress completed");
        } catch (error) {
          console.error("❌ Error in startCompilationWithProgress:", error);
        }
      }
    );
    console.log("✅ ready-for-compile listener set up");

    // Test: Listen to specific events to debug what's happening
    console.log("🔍 Setting up debug listeners for specific events...");
    try {
      // Listen for a few common events to see if any events are coming through
      const debugEvents = [
        "ready-for-compile",
        "preview-url-updated",
        "compilation-progress",
      ];

      for (const eventName of debugEvents) {
        await listen(eventName, (event) => {
          console.log(`🔍 DEBUG: Received ${eventName} event:`, event.payload);
        });
        console.log(`✅ Debug listener for ${eventName} set up`);
      }
    } catch (error) {
      console.error("❌ Failed to set up debug listeners:", error);
    }

    // Listen for preview URL updates from backend
    await listen("preview-url-updated", (event) => {
      console.log("📺 Received preview URL update:", event.payload);

      const { url, folder_path } = event.payload;

      if (url) {
        // Update the iframe source
        loadPreview(url);

        console.log(`✅ Preview updated: ${folder_path} → ${url}`);
      }
    });

    console.log("✅ Preview event listeners setup complete");
  } catch (error) {
    console.error("❌ Failed to setup preview event listeners:", error);
  }
}
