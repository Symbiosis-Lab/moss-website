document.addEventListener('DOMContentLoaded', async () => {
  console.log('Moss app loaded')
  
  // Test the Tauri backend connection
  const { invoke } = await import('@tauri-apps/api/core')
  
  try {
    const greeting = await invoke('greet', { name: 'Moss' })
    console.log('Backend connected:', greeting)
    
    // Test tray icon functionality
    console.log('Testing tray icon...')
    try {
      const trayResult = await invoke('test_tray_icon')
      console.log('✅ Tray icon test:', trayResult)
    } catch (trayError) {
      console.error('❌ Tray icon test failed:', trayError)
    }
  } catch (error) {
    console.error('Backend connection failed:', error)
  }
})