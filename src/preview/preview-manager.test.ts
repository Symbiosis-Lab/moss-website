// Preview Manager Unit Tests
// Tests core functionality of preview system including initialization,
// progress handling, and state management

import { describe, test, expect, beforeEach, afterEach, vi } from 'vitest';
import { PreviewManager } from './preview-manager';
import type { ProgressUpdate } from '../bindings';

// Mock the backend integration
vi.mock('../backend/tauri-integration', () => ({
  getCompileConfig: vi.fn(),
  compileWithProgress: vi.fn(),
}));

// Mock progress templates
vi.mock('./progress-templates', () => ({
  progressTemplates: {
    welcome: vi.fn().mockReturnValue('<div>Welcome</div>'),
    loading: vi.fn().mockReturnValue('<div>Loading</div>'),
    completed: vi.fn().mockReturnValue('<div>Completed</div>'),
    withSteps: vi.fn().mockReturnValue('<div>Steps</div>'),
  },
}));

// Import mocked functions
import { getCompileConfig, compileWithProgress } from '../backend/tauri-integration';
import { progressTemplates } from './progress-templates';

describe('PreviewManager', () => {
  let previewManager: PreviewManager;
  let mockIframe: HTMLIFrameElement;

  beforeEach(() => {
    // Create mock iframe element
    mockIframe = {
      src: '',
      srcdoc: '',
      removeAttribute: vi.fn(),
      onerror: null,
    } as any;

    // Mock document.getElementById to return our mock iframe
    vi.spyOn(document, 'getElementById').mockReturnValue(mockIframe);

    // Mock global URL and Blob
    global.URL = {
      createObjectURL: vi.fn().mockReturnValue('blob:mock-url'),
    } as any;
    global.Blob = vi.fn() as any;

    // Clear all mocks
    vi.clearAllMocks();

    // Re-setup progress template mocks after clearing
    (progressTemplates.welcome as any).mockReturnValue('<div>Welcome</div>');
    (progressTemplates.loading as any).mockReturnValue('<div>Loading</div>');
    (progressTemplates.completed as any).mockReturnValue('<div>Completed</div>');
    (progressTemplates.withSteps as any).mockReturnValue('<div>Steps</div>');
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  describe('Constructor', () => {
    test('creates instance with default iframe id', () => {
      previewManager = new PreviewManager();
      expect(document.getElementById).toHaveBeenCalledWith('moss-preview-iframe');
    });

    test('creates instance with custom iframe id', () => {
      previewManager = new PreviewManager('custom-iframe');
      expect(document.getElementById).toHaveBeenCalledWith('custom-iframe');
    });

    test('throws error when iframe not found', () => {
      vi.spyOn(document, 'getElementById').mockReturnValue(null);

      expect(() => new PreviewManager()).toThrow(
        "Preview iframe with id 'moss-preview-iframe' not found"
      );
    });
  });

  describe('Configuration and Initialization', () => {
    beforeEach(() => {
      previewManager = new PreviewManager();
    });

    test('getCompileConfig returns config when window.__MOSS_COMPILE_CONFIG__ exists', () => {
      // Mock the actual getCompileConfig function behavior
      const mockConfig = { folder_path: '/test/path', auto_serve: true };
      (getCompileConfig as any).mockReturnValue(mockConfig);

      const result = getCompileConfig();
      expect(result).toEqual(mockConfig);
    });

    test('getCompileConfig returns null when no config exists', () => {
      (getCompileConfig as any).mockReturnValue(null);

      const result = getCompileConfig();
      expect(result).toBeNull();
    });

    test('initialize calls startCompilation when config exists', async () => {
      const mockConfig = { folder_path: '/test/folder', auto_serve: true };
      (getCompileConfig as any).mockReturnValue(mockConfig);
      (compileWithProgress as any).mockResolvedValue('success');

      const startCompilationSpy = vi.spyOn(previewManager, 'startCompilation');

      await previewManager.initialize();

      expect(getCompileConfig).toHaveBeenCalled();
      expect(startCompilationSpy).toHaveBeenCalledWith('/test/folder', true);
    });

    test('initialize shows welcome message when no config', async () => {
      (getCompileConfig as any).mockReturnValue(null);

      const showMessageSpy = vi.spyOn(previewManager, 'showMessage');

      await previewManager.initialize();

      expect(getCompileConfig).toHaveBeenCalled();
      expect(showMessageSpy).toHaveBeenCalledWith(
        'Welcome to moss! Right-click on a folder to publish.'
      );
    });
  });

  describe('Progress Event Handling', () => {
    beforeEach(() => {
      previewManager = new PreviewManager();
    });

    test('handleProgressUpdate updates progress display', () => {
      const update: ProgressUpdate = {
        step: 'scanning',
        message: 'Scanning files...',
        percentage: 25,
        completed: false,
        port: null,
      };

      const showProgressSpy = vi.spyOn(previewManager, 'showProgress');

      // Call private method via bracket notation for testing
      (previewManager as any).handleProgressUpdate(update);

      expect(showProgressSpy).toHaveBeenCalledWith(
        'Scanning files...',
        25,
        true // !update.completed
      );
    });

    test('handleProgressUpdate loads preview when complete with port', () => {
      const update: ProgressUpdate = {
        step: 'serving',
        message: 'Server started',
        percentage: 100,
        completed: true,
        port: 3000,
      };

      const loadPreviewSpy = vi.spyOn(previewManager, 'loadPreview');
      vi.useFakeTimers();

      (previewManager as any).handleProgressUpdate(update);

      // Fast-forward the setTimeout
      vi.advanceTimersByTime(1000);

      expect(loadPreviewSpy).toHaveBeenCalledWith('http://localhost:3000');

      vi.useRealTimers();
    });

    test('handleProgressUpdate does not load preview when complete without port', () => {
      const update: ProgressUpdate = {
        step: 'complete',
        message: 'Compilation complete',
        percentage: 100,
        completed: true,
        port: null,
      };

      const loadPreviewSpy = vi.spyOn(previewManager, 'loadPreview');
      vi.useFakeTimers();

      (previewManager as any).handleProgressUpdate(update);
      vi.advanceTimersByTime(1000);

      expect(loadPreviewSpy).not.toHaveBeenCalled();

      vi.useRealTimers();
    });

    test('getPreviewUrl returns correct URL when port exists', () => {
      const update: ProgressUpdate = {
        step: 'serving',
        message: 'Server started',
        percentage: 100,
        completed: true,
        port: 4000,
      };

      const result = (previewManager as any).getPreviewUrl(update);
      expect(result).toBe('http://localhost:4000');
    });

    test('getPreviewUrl returns null when no port', () => {
      const update: ProgressUpdate = {
        step: 'complete',
        message: 'Complete',
        percentage: 100,
        completed: true,
        port: null,
      };

      const result = (previewManager as any).getPreviewUrl(update);
      expect(result).toBeNull();
    });
  });

  describe('UI State Management', () => {
    beforeEach(() => {
      previewManager = new PreviewManager();
    });

    test('showMessage displays welcome message', () => {
      previewManager.showMessage('Welcome test message');

      expect(progressTemplates.welcome).toHaveBeenCalledWith('Welcome test message');
      expect(global.Blob).toHaveBeenCalledWith(['<div>Welcome</div>'], { type: 'text/html' });
      expect(global.URL.createObjectURL).toHaveBeenCalled();
      expect(mockIframe.src).toBe('blob:mock-url');
    });

    test('showMessage displays message with steps', () => {
      const details = { steps: ['Step 1', 'Step 2'] };

      previewManager.showMessage('Processing...', false, details);

      expect(progressTemplates.withSteps).toHaveBeenCalledWith('Processing...', ['Step 1', 'Step 2']);
      expect(mockIframe.src).toBe('blob:mock-url');
    });

    test('showProgress displays loading state when active', () => {
      previewManager.showProgress('Loading...', 50, true);

      expect(progressTemplates.loading).toHaveBeenCalledWith('Loading...', 50);
      expect(mockIframe.srcdoc).toBe('<div>Loading</div>');
    });

    test('showProgress displays completed state when inactive', () => {
      previewManager.showProgress('Done!', 100, false);

      expect(progressTemplates.completed).toHaveBeenCalledWith('Done!', 100);
      expect(mockIframe.srcdoc).toBe('<div>Completed</div>');
    });

    test('loadPreview sets iframe src and removes srcdoc', () => {
      const testUrl = 'http://localhost:3000';

      previewManager.loadPreview(testUrl);

      expect(mockIframe.src).toBe(testUrl);
      expect(mockIframe.removeAttribute).toHaveBeenCalledWith('srcdoc');
    });

    test('loadPreview sets up error handler', () => {
      const showMessageSpy = vi.spyOn(previewManager, 'showMessage');

      previewManager.loadPreview('http://localhost:3000');

      // Simulate iframe error
      mockIframe.onerror?.({} as any);

      expect(showMessageSpy).toHaveBeenCalledWith('Failed to load preview');
    });

    test('refresh reloads iframe when src is not about:blank', () => {
      mockIframe.src = 'http://localhost:3000';

      previewManager.refresh();

      expect(mockIframe.src).toBe('http://localhost:3000');
    });

    test('refresh does nothing when iframe src is about:blank', () => {
      mockIframe.src = 'about:blank';
      const originalSrc = mockIframe.src;

      previewManager.refresh();

      // Should not change
      expect(mockIframe.src).toBe(originalSrc);
    });
  });

  describe('Compilation Flow', () => {
    beforeEach(() => {
      previewManager = new PreviewManager();
    });

    test('startCompilation shows initial progress and calls compileWithProgress', async () => {
      const showProgressSpy = vi.spyOn(previewManager, 'showProgress');
      (compileWithProgress as any).mockResolvedValue('success');

      await previewManager.startCompilation('/test/folder', true);

      expect(showProgressSpy).toHaveBeenCalledWith('Preparing to compile...', 0, true);
      expect(compileWithProgress).toHaveBeenCalledWith(
        '/test/folder',
        true,
        expect.any(Function)
      );
    });

    test('startCompilation handles compilation errors', async () => {
      const showMessageSpy = vi.spyOn(previewManager, 'showMessage');
      const error = new Error('Compilation failed');
      (compileWithProgress as any).mockRejectedValue(error);

      await previewManager.startCompilation('/test/folder', true);

      expect(showMessageSpy).toHaveBeenCalledWith('Failed to compile: Error: Compilation failed');
    });

    test('startCompilation uses default autoServe value', async () => {
      (compileWithProgress as any).mockResolvedValue('success');

      await previewManager.startCompilation('/test/folder');

      expect(compileWithProgress).toHaveBeenCalledWith(
        '/test/folder',
        true, // default value
        expect.any(Function)
      );
    });
  });

  describe('Panel Manager Integration', () => {
    beforeEach(() => {
      previewManager = new PreviewManager();
    });

    test('setPanelManager stores panel manager reference', () => {
      const mockPanelManager = { someMethod: vi.fn() };

      previewManager.setPanelManager(mockPanelManager);

      expect((previewManager as any).panelManager).toBe(mockPanelManager);
    });
  });
});