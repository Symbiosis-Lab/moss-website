document.addEventListener('DOMContentLoaded', async () => {
  console.log('Moss app loaded')
  
  // Test the Tauri backend connection
  const { invoke } = await import('@tauri-apps/api/core')
  const { getCurrent } = await import('@tauri-apps/plugin-deep-link')
  
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

  // Handle deep links (e.g., moss://publish?path=/path/to/folder)
  try {
    const urls = await getCurrent()
    if (urls && urls.length > 0) {
      console.log('🔗 App started via deep link:', urls)
      for (const url of urls) {
        await handleDeepLink(url)
      }
    }
  } catch (error) {
    console.log('No deep link on startup (this is normal)')
  }

  // Check if this is first launch - install Finder integration
  try {
    console.log('🔧 Checking Finder integration...')
    const integrationResult = await invoke('install_finder_integration')
    console.log('✅ Finder integration:', integrationResult)
  } catch (error) {
    console.error('❌ Failed to install Finder integration:', error)
  }

  // TODO: Settings UI will be implemented here
})

async function handleDeepLink(url) {
  console.log('Processing deep link:', url)
  
  try {
    const urlObj = new URL(url)
    
    // Handle moss://publish?path=/path/to/folder
    if (urlObj.protocol === 'moss:' && urlObj.pathname === '//publish') {
      const folderPath = urlObj.searchParams.get('path')
      if (folderPath) {
        console.log('📁 Publishing folder via deep link:', folderPath)
        
        const { invoke } = await import('@tauri-apps/api/core')
        try {
          const result = await invoke('publish_folder_from_deep_link', {
            folderPath: decodeURIComponent(folderPath)
          })
          console.log('✅ Publish result:', result)
        } catch (error) {
          console.error('❌ Publish failed:', error)
        }
      } else {
        console.error('❌ No folder path in deep link')
      }
    } else {
      console.log('ℹ️ Unknown deep link format:', url)
    }
  } catch (error) {
    console.error('❌ Failed to parse deep link:', error)
  }
}