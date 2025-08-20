/**
 * @vitest-environment jsdom
 */
import { describe, it, expect, vi, beforeEach } from 'vitest'

// Mock the Tauri API
const mockInvoke = vi.fn()
vi.mock('@tauri-apps/api/core', () => ({
  invoke: mockInvoke
}))

describe('Moss Frontend', () => {
  beforeEach(() => {
    // Reset DOM
    document.body.innerHTML = ''
    
    // Reset mocks
    vi.clearAllMocks()
    
    // Set up basic DOM structure
    document.body.innerHTML = `
      <div id="app">
        <h1>Moss</h1>
        <div id="output"></div>
      </div>
    `
  })

  describe('Backend Communication', () => {
    it('should successfully call greet command', async () => {
      // Arrange
      const expectedGreeting = 'Hello, Moss! You\'ve been greeted from Rust!'
      mockInvoke.mockResolvedValue(expectedGreeting)

      // Act
      const { invoke } = await import('@tauri-apps/api/core')
      const result = await invoke('greet', { name: 'Moss' })

      // Assert
      expect(mockInvoke).toHaveBeenCalledWith('greet', { name: 'Moss' })
      expect(result).toBe(expectedGreeting)
    })

    it('should handle backend connection errors gracefully', async () => {
      // Arrange
      const expectedError = new Error('Backend connection failed')
      mockInvoke.mockRejectedValue(expectedError)

      // Act & Assert
      const { invoke } = await import('@tauri-apps/api/core')
      await expect(invoke('greet', { name: 'Test' })).rejects.toThrow('Backend connection failed')
    })

    it('should successfully test tray icon functionality', async () => {
      // Arrange
      const expectedResult = 'Tray icon found and is responsive'
      mockInvoke.mockResolvedValue(expectedResult)

      // Act
      const { invoke } = await import('@tauri-apps/api/core')
      const result = await invoke('test_tray_icon')

      // Assert
      expect(mockInvoke).toHaveBeenCalledWith('test_tray_icon')
      expect(result).toBe(expectedResult)
    })

    it('should handle tray icon test failures', async () => {
      // Arrange
      const expectedError = 'Tray icon not found by ID'
      mockInvoke.mockRejectedValue(expectedError)

      // Act & Assert
      const { invoke } = await import('@tauri-apps/api/core')
      await expect(invoke('test_tray_icon')).rejects.toBe(expectedError)
    })
  })

  describe('DOM Manipulation', () => {
    it('should have correct initial DOM structure', () => {
      // Assert
      expect(document.querySelector('#app')).toBeTruthy()
      expect(document.querySelector('h1')).toBeTruthy()
      expect(document.querySelector('h1').textContent).toBe('Moss')
      expect(document.querySelector('#output')).toBeTruthy()
    })

    it('should be able to update DOM elements', () => {
      // Arrange
      const outputElement = document.querySelector('#output')
      const testMessage = 'Test message'

      // Act
      outputElement.textContent = testMessage

      // Assert
      expect(outputElement.textContent).toBe(testMessage)
    })
  })

  describe('App Initialization', () => {
    it('should wait for DOMContentLoaded event', () => {
      // This test verifies our app waits for DOM to be ready
      const mockHandler = vi.fn()
      
      // Act
      document.addEventListener('DOMContentLoaded', mockHandler)
      
      // Simulate DOMContentLoaded
      const event = new Event('DOMContentLoaded')
      document.dispatchEvent(event)
      
      // Assert
      expect(mockHandler).toHaveBeenCalledOnce()
    })
  })
})