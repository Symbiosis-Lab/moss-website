//! Preview server management for compiled websites
//! 
//! This module provides a lightweight HTTP server for previewing generated static sites.
//! It uses Axum and tower-http to serve files with proper headers, avoiding CORS issues
//! and providing real web server behavior for accurate preview.

use axum::Router;
use tower_http::services::ServeDir;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use std::path::Path;
use crate::types::ServerState;
use std::sync::Mutex;
use tauri::Manager;

/// Starts a local HTTP preview server for the generated website.
///
/// Creates a lightweight Axum server to properly serve the static files with correct
/// HTTP headers, avoiding CORS issues and providing real web server behavior.
/// Supports both CLI mode (no reuse) and GUI mode (with server reuse).
///
/// # Arguments
/// * `site_path` - Path to the generated site directory containing index.html
/// * `app_handle` - Optional Tauri app handle for server reuse (None for CLI mode)
///
/// # Returns
/// * `Ok(u16)` - Successfully started server, returns the port number used
/// * `Err(String)` - Failed to start server
///
/// # Server Reuse (GUI Mode)
/// When `app_handle` is provided:
/// - Checks if server already exists for the same folder path
/// - Reuses existing server if it's still responding
/// - Only creates new server if folder path changes or server is down
/// - Records new servers in app state for future reuse
///
/// # CLI Mode
/// When `app_handle` is None:
/// - Creates server directly without state management
/// - No server reuse logic
/// - Suitable for command-line usage
///
/// # Implementation
/// - Uses Axum + tower-http for lightweight static file serving
/// - Finds an available port automatically (starting from 8080)
/// - Spawns server in background using existing Tokio runtime
/// - Falls back to thread-based runtime for test environments
/// - Uses localhost (127.0.0.1) for security
///
/// # Port Selection
/// - Tries port 8080 first (consistency with preview window)
/// - Scans for next available port if 8080 is taken
/// - Fails if no ports available in range
pub async fn start_preview_server(
    site_path: &str,
    app_handle: Option<&tauri::AppHandle>
) -> Result<u16, String> {
    // If app handle provided, check for existing server to reuse
    if let Some(app) = app_handle {
        let canonical_path = std::fs::canonicalize(site_path)
            .map_err(|e| format!("Failed to resolve site path: {}", e))?
            .to_string_lossy()
            .to_string();

        // Check for existing server
        let existing_port = if let Some(server_state) = app.try_state::<Mutex<ServerState>>() {
            if let Ok(state) = server_state.lock() {
                state.active_servers.get(&canonical_path).copied()
            } else {
                None
            }
        } else {
            None
        };

        if let Some(port) = existing_port {
            // Verify existing server is still responding
            if verify_server_ready(port).await.is_ok() {
                println!("🔄 Reusing existing server on port {} for {}", port, canonical_path);
                return Ok(port);
            } else {
                println!("🔧 Existing server on port {} not responding, creating new one", port);
            }
        }

        // Create new server
        let port = create_new_server(site_path).await?;

        // Record new server in state
        if let Some(server_state) = app.try_state::<Mutex<ServerState>>() {
            if let Ok(mut state) = server_state.lock() {
                state.active_servers.insert(canonical_path, port);
                println!("🔧 Recorded new server on port {} in state", port);
            }
        }

        Ok(port)
    } else {
        // No app handle - create server directly (CLI mode)
        create_new_server(site_path).await
    }
}

/// Internal function to create a new preview server without state management
async fn create_new_server(site_path: &str) -> Result<u16, String> {
    let site_path = site_path.to_string();

    // === SETUP PHASE ===
    // Check if index.html exists
    let index_path = Path::new(&site_path).join("index.html");
    if !index_path.exists() {
        return Err("Generated site has no index.html file".to_string());
    }
    
    // Use port 8080 for consistency with preview window
    let port = if is_port_available(8080) { 8080 } else { find_available_port(8080)? };
    
    // === EXECUTION PHASE ===
    // Start server in background
    let server_site_path = site_path.clone();
    
    // Try to spawn in existing Tokio runtime, fallback to thread for tests
    if let Ok(handle) = tokio::runtime::Handle::try_current() {
        handle.spawn(async move {
            // Create and run Axum server inline
            let app = Router::new()
                .fallback_service(ServeDir::new(server_site_path));
            
            let addr = SocketAddr::from(([127, 0, 0, 1], port));
            if let Ok(listener) = TcpListener::bind(&addr).await {
                if let Err(e) = axum::serve(listener, app).await {
                    eprintln!("❌ Preview server error: {}", e);
                }
            } else {
                eprintln!("❌ Failed to bind to port {}", port);
            }
        });
    } else {
        // Fallback for test environment - create minimal runtime
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                // Create and run Axum server inline
                let app = Router::new()
                    .fallback_service(ServeDir::new(server_site_path));
                
                let addr = SocketAddr::from(([127, 0, 0, 1], port));
                if let Ok(listener) = TcpListener::bind(&addr).await {
                    if let Err(e) = axum::serve(listener, app).await {
                        eprintln!("❌ Preview server error: {}", e);
                    }
                } else {
                    eprintln!("❌ Failed to bind to port {}", port);
                }
            });
        });
    }
    
    // Verify server is ready before returning
    println!("🔧 Verifying server is ready on port {}", port);
    if verify_server_ready(port).await.is_err() {
        return Err(format!("Server started but failed readiness check on port {}", port));
    }
    
    println!("✅ Preview server ready on http://localhost:{}", port);
    Ok(port)
}

/// Finds an available TCP port starting from the given port number.
/// 
/// Scans through a range of 100 ports starting from `start_port` to find
/// an available port for the preview server.
/// 
/// # Arguments
/// * `start_port` - Port number to start scanning from
/// 
/// # Returns
/// * `Ok(u16)` - First available port found
/// * `Err(String)` - No available ports in the scanned range
pub fn find_available_port(start_port: u16) -> Result<u16, String> {
    for port in start_port..start_port + 100 {
        if is_port_available(port) {
            return Ok(port);
        }
    }
    Err(format!("No available ports found starting from {}", start_port))
}

/// Checks if a TCP port is available for binding.
/// 
/// Attempts to bind to the specified port on localhost to determine
/// if it's available for use by the preview server.
/// 
/// # Arguments
/// * `port` - Port number to test
/// 
/// # Returns
/// * `true` - Port is available
/// * `false` - Port is already in use
pub fn is_port_available(port: u16) -> bool {
    std::net::TcpListener::bind(("127.0.0.1", port)).is_ok()
}

/// Verifies that the preview server is ready and responding on the given port.
/// 
/// Makes an HTTP request to the server to ensure it's actually serving content
/// before reporting success to the caller.
/// 
/// # Arguments
/// * `port` - Port number where server should be running
/// 
/// # Returns
/// * `Ok(())` - Server is ready and responding
/// * `Err(String)` - Server is not ready or responding
async fn verify_server_ready(port: u16) -> Result<(), String> {
    let url = format!("http://localhost:{}", port);
    let max_attempts = 10;
    let delay_ms = 200;
    
    for attempt in 1..=max_attempts {
        // Give server time to start up
        tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;
        
        // Try to make a simple HTTP request
        match reqwest::get(&url).await {
            Ok(response) => {
                if response.status().is_success() {
                    println!("🔧 Server readiness check passed on attempt {}", attempt);
                    return Ok(());
                } else {
                    println!("🔧 Server responded with status {} on attempt {}", response.status(), attempt);
                }
            }
            Err(e) => {
                println!("🔧 Server readiness check attempt {}/{}: {}", attempt, max_attempts, e);
            }
        }
    }
    
    Err(format!("Server failed to respond after {} attempts", max_attempts))
}

/// Stops a preview server running on the given port
///
/// Uses system commands to kill any process using the specified port.
/// This is a simple but effective approach that works across platforms.
///
/// # Arguments
/// * `port` - Port number of the server to stop
///
/// # Returns
/// * `Ok(())` - Server stopped successfully (or no server was running)
/// * `Err(String)` - Failed to stop server
pub fn stop_preview_server(port: u16) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;

        // Use lsof to find and kill processes using the port
        let output = Command::new("lsof")
            .args(["-ti", &format!(":{}", port)])
            .output()
            .map_err(|e| format!("Failed to run lsof: {}", e))?;

        if output.status.success() {
            let pids = String::from_utf8_lossy(&output.stdout);
            let pids: Vec<&str> = pids.trim().split('\n').filter(|pid| !pid.is_empty()).collect();

            if !pids.is_empty() {
                println!("🔧 Stopping preview server on port {} (PIDs: {:?})", port, pids);

                for pid in pids {
                    let kill_result = Command::new("kill")
                        .arg(pid)
                        .output()
                        .map_err(|e| format!("Failed to kill process {}: {}", pid, e))?;

                    if !kill_result.status.success() {
                        eprintln!("⚠️ Failed to kill process {}", pid);
                    }
                }

                println!("✅ Preview server stopped on port {}", port);
            } else {
                println!("ℹ️ No server running on port {}", port);
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        use std::process::Command;

        // Use lsof to find and kill processes using the port
        let output = Command::new("lsof")
            .args(["-ti", &format!(":{}", port)])
            .output()
            .map_err(|e| format!("Failed to run lsof: {}", e))?;

        if output.status.success() {
            let pids = String::from_utf8_lossy(&output.stdout);
            let pids: Vec<&str> = pids.trim().split('\n').filter(|pid| !pid.is_empty()).collect();

            if !pids.is_empty() {
                println!("🔧 Stopping preview server on port {} (PIDs: {:?})", port, pids);

                for pid in pids {
                    let kill_result = Command::new("kill")
                        .arg(pid)
                        .output()
                        .map_err(|e| format!("Failed to kill process {}: {}", pid, e))?;

                    if !kill_result.status.success() {
                        eprintln!("⚠️ Failed to kill process {}", pid);
                    }
                }

                println!("✅ Preview server stopped on port {}", port);
            } else {
                println!("ℹ️ No server running on port {}", port);
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        use std::process::Command;

        // Use netstat and taskkill on Windows
        let output = Command::new("netstat")
            .args(["-ano"])
            .output()
            .map_err(|e| format!("Failed to run netstat: {}", e))?;

        if output.status.success() {
            let netstat_output = String::from_utf8_lossy(&output.stdout);

            // Find lines containing our port
            for line in netstat_output.lines() {
                if line.contains(&format!(":{}", port)) && line.contains("LISTENING") {
                    // Extract PID (last column)
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if let Some(pid) = parts.last() {
                        println!("🔧 Stopping preview server on port {} (PID: {})", port, pid);

                        let kill_result = Command::new("taskkill")
                            .args(["/F", "/PID", pid])
                            .output()
                            .map_err(|e| format!("Failed to kill process {}: {}", pid, e))?;

                        if kill_result.status.success() {
                            println!("✅ Preview server stopped on port {}", port);
                        } else {
                            eprintln!("⚠️ Failed to kill process {}", pid);
                        }
                        break;
                    }
                }
            }
        }
    }

    Ok(())
}