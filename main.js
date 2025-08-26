document.addEventListener('DOMContentLoaded', async () => {
  console.log('Moss app loaded')
  
  // Get Tauri API functions
  const { invoke } = await import('@tauri-apps/api/core')
  const { getCurrent, onOpenUrl } = await import('@tauri-apps/plugin-deep-link')
  
  // Test backend connection with actual commands
  try {
    console.log('Testing backend connection...')
    const systemStatus = await invoke('get_system_status')
    console.log('✅ Backend connected:', systemStatus)
  } catch (error) {
    console.error('❌ Backend connection failed:', error)
  }

  // Add test button for publishing
  const testButton = document.createElement('button');
  testButton.textContent = '🧪 Test Publish';
  testButton.onclick = async () => {
    try {
      const result = await invoke('test_publish_command', { 
        folderPath: '/Users/liuguo/repo/moss/test-content/simple-blog' 
      });
      console.log('✅ Test publish result:', result);
      alert('Published successfully! Check browser for preview.');
    } catch (error) {
      console.error('❌ Test publish failed:', error);
      alert('Publish failed: ' + error);
    }
  };
  document.body.appendChild(testButton);

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

  // Listen for deep links while app is running
  try {
    console.log('🎯 Setting up runtime deep link listener...')
    await onOpenUrl(async (urls) => {
      console.log('🔗 Runtime deep link received:', urls)
      for (const url of urls) {
        await handleDeepLink(url)
      }
    })
    console.log('✅ Runtime deep link listener active')
  } catch (error) {
    console.error('❌ Failed to set up runtime deep link listener:', error)
  }

  // Check if this is first launch - install Finder integration
  try {
    console.log('🔧 Checking Finder integration...')
    const integrationResult = await invoke('install_finder_integration')
    console.log('✅ Finder integration:', integrationResult)
  } catch (error) {
    console.error('❌ Failed to install Finder integration:', error)
  }
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
          const result = await invoke('publish_folder', {
            folder_path: decodeURIComponent(folderPath)
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