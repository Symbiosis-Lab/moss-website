// moss Tauri App - Main Frontend Entry Point
// Lean entry point that coordinates all subsystems

import { mossApp } from "./app";

document.addEventListener("DOMContentLoaded", async () => {
  console.log("🌿 moss Tauri app loading...");

  try {
    // Initialize the complete moss application
    await mossApp.initialize();
  } catch (error) {
    console.error("❌ Failed to initialize moss app:", error);

    // Show error message to user
    const body = document.body;
    if (body) {
      body.innerHTML = `
        <div style="
          display: flex;
          align-items: center;
          justify-content: center;
          height: 100vh;
          font-family: -apple-system, BlinkMacSystemFont, sans-serif;
          background: #f8f9fa;
          color: #ef4444;
          text-align: center;
          padding: 2rem;
        ">
          <div>
            <div style="font-size: 48px; margin-bottom: 1rem;">⚠️</div>
            <h2>Failed to initialize moss</h2>
            <p>Please reload the application or check the console for details.</p>
            <button onclick="window.location.reload()" style="
              padding: 8px 16px;
              background: #6366f1;
              color: white;
              border: none;
              border-radius: 6px;
              cursor: pointer;
              margin-top: 1rem;
            ">
              Reload
            </button>
          </div>
        </div>
      `;
    }
  }
});
