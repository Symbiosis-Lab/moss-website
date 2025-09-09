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

/// Starts a local HTTP preview server for the generated website.
/// 
/// Creates a lightweight Axum server to properly serve the static files with correct
/// HTTP headers, avoiding CORS issues and providing real web server behavior.
/// 
/// # Arguments
/// * `site_path` - Path to the generated site directory containing index.html
/// 
/// # Returns
/// * `Ok(())` - Successfully started server
/// * `Err(String)` - Failed to start server
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
pub fn start_preview_server(site_path: &str) -> Result<(), String> {
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
    
    // Give server a moment to start
    std::thread::sleep(std::time::Duration::from_millis(500));
    
    Ok(())
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