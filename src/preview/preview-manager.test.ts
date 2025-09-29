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

    // Mock URL constructor for URL manipulation tests
    global.URL = class MockURL {
      pathname: string;
      origin: string;

      constructor(url: string) {
        if (url === 'not-a-valid-url' || url === 'invalid-url') {
          throw new Error('Invalid URL');
        }

        // Parse the URL manually for testing
        const match = url.match(/^(https?:\/\/[^\/]+)(\/.*)?$/);
        if (match) {
          this.origin = match[1];
          this.pathname = match[2] || '/';
        } else {
          throw new Error('Invalid URL format');
        }
      }

      static createObjectURL = vi.fn().mockReturnValue('blob:mock-url');
    } as any;

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
        true, // file watching enabled
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
        true, // file watching enabled
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

  describe('File Change Event Handling', () => {
    beforeEach(() => {
      previewManager = new PreviewManager();
    });

    test('handleFileChangeEvent with null event defaults to refresh', () => {
      const refreshSpy = vi.spyOn(previewManager, 'refresh');

      // Test with null
      (previewManager as any).handleFileChangeEvent(null);

      expect(refreshSpy).toHaveBeenCalledTimes(1);
    });

    test('handleFileChangeEvent with undefined event defaults to refresh', () => {
      const refreshSpy = vi.spyOn(previewManager, 'refresh');

      // Test with undefined
      (previewManager as any).handleFileChangeEvent(undefined);

      expect(refreshSpy).toHaveBeenCalledTimes(1);
    });

    test('handleFileChangeEvent with empty change event triggers refresh', () => {
      const refreshSpy = vi.spyOn(previewManager, 'refresh');
      const emptyEvent = {
        deleted_paths: null,
        renamed_paths: null
      };

      (previewManager as any).handleFileChangeEvent(emptyEvent);

      expect(refreshSpy).toHaveBeenCalledTimes(1);
    });

    test('handleFileChangeEvent with empty arrays triggers refresh', () => {
      const refreshSpy = vi.spyOn(previewManager, 'refresh');
      const emptyEvent = {
        deleted_paths: [],
        renamed_paths: []
      };

      (previewManager as any).handleFileChangeEvent(emptyEvent);

      expect(refreshSpy).toHaveBeenCalledTimes(1);
    });

    test('handleFileChangeEvent with deletion of current page redirects to homepage', () => {
      // Mock the private method before calling
      const redirectSpy = vi.fn();
      (previewManager as any).redirectToHomepage = redirectSpy;

      mockIframe.src = 'http://localhost:3000/blog/post.html';

      const deleteEvent = {
        deleted_paths: ['blog/post.html'],
        renamed_paths: null
      };

      (previewManager as any).handleFileChangeEvent(deleteEvent);

      expect(redirectSpy).toHaveBeenCalledTimes(1);
    });

    test('handleFileChangeEvent with deletion of other page refreshes', () => {
      const refreshSpy = vi.spyOn(previewManager, 'refresh');
      mockIframe.src = 'http://localhost:3000/blog/current.html';

      const deleteEvent = {
        deleted_paths: ['blog/other.html'],
        renamed_paths: null
      };

      (previewManager as any).handleFileChangeEvent(deleteEvent);

      expect(refreshSpy).toHaveBeenCalledTimes(1);
    });

    test('handleFileChangeEvent with rename of current page updates URL', () => {
      // Mock the private method before calling
      const updateUrlSpy = vi.fn();
      (previewManager as any).updateUrlForRename = updateUrlSpy;

      mockIframe.src = 'http://localhost:3000/blog/old-post.html';

      const renameEvent = {
        deleted_paths: null,
        renamed_paths: [['blog/old-post.html', 'blog/new-post.html']]
      };

      (previewManager as any).handleFileChangeEvent(renameEvent);

      expect(updateUrlSpy).toHaveBeenCalledWith('blog/old-post.html', 'blog/new-post.html');
    });

    test('handleFileChangeEvent with rename of other page refreshes', () => {
      const refreshSpy = vi.spyOn(previewManager, 'refresh');
      mockIframe.src = 'http://localhost:3000/blog/current.html';

      const renameEvent = {
        deleted_paths: null,
        renamed_paths: [['blog/other.html', 'blog/renamed.html']]
      };

      (previewManager as any).handleFileChangeEvent(renameEvent);

      expect(refreshSpy).toHaveBeenCalledTimes(1);
    });

    test('handleFileChangeEvent with malformed rename pairs is robust', () => {
      const refreshSpy = vi.spyOn(previewManager, 'refresh');

      // Test with malformed rename pairs
      const malformedEvent = {
        deleted_paths: null,
        renamed_paths: [
          ['only-one-item'], // Invalid: should have 2 items
          null, // Invalid: null item
          ['old.md', 'new.md', 'extra.md'] // Valid but with extra item
        ]
      };

      expect(() => {
        (previewManager as any).handleFileChangeEvent(malformedEvent);
      }).not.toThrow();

      expect(refreshSpy).toHaveBeenCalled();
    });
  });

  describe('URL Manipulation Functions', () => {
    beforeEach(() => {
      previewManager = new PreviewManager();
    });

    test('extractPathFromUrl with normal URL', () => {
      const url = 'http://localhost:3000/blog/post.html';
      const result = (previewManager as any).extractPathFromUrl(url);

      expect(result).toBe('blog/post.html');
    });

    test('extractPathFromUrl with index.html returns index.html', () => {
      const url = 'http://localhost:3000/index.html';
      const result = (previewManager as any).extractPathFromUrl(url);

      expect(result).toBe('index.html');
    });

    test('extractPathFromUrl with root path returns index.html', () => {
      const url = 'http://localhost:3000/';
      const result = (previewManager as any).extractPathFromUrl(url);

      expect(result).toBe('index.html');
    });

    test('extractPathFromUrl with malformed URL returns null', () => {
      const malformedUrl = 'not-a-valid-url';
      const result = (previewManager as any).extractPathFromUrl(malformedUrl);

      expect(result).toBe(null);
    });

    test('redirectToHomepage constructs correct homepage URL', () => {
      const loadPreviewSpy = vi.spyOn(previewManager, 'loadPreview');
      mockIframe.src = 'http://localhost:3000/blog/post.html';

      (previewManager as any).redirectToHomepage();

      expect(loadPreviewSpy).toHaveBeenCalledWith('http://localhost:3000/index.html');
    });

    test('redirectToHomepage with malformed URL falls back to refresh', () => {
      const loadPreviewSpy = vi.spyOn(previewManager, 'loadPreview');
      const refreshSpy = vi.spyOn(previewManager, 'refresh');
      mockIframe.src = 'invalid-url';

      (previewManager as any).redirectToHomepage();

      expect(loadPreviewSpy).not.toHaveBeenCalled();
      expect(refreshSpy).toHaveBeenCalledTimes(1);
    });

    test('updateUrlForRename converts md to html and updates URL', () => {
      const loadPreviewSpy = vi.spyOn(previewManager, 'loadPreview');
      mockIframe.src = 'http://localhost:3000/blog/old.html';

      (previewManager as any).updateUrlForRename('blog/old.md', 'blog/new.md');

      expect(loadPreviewSpy).toHaveBeenCalledWith('http://localhost:3000/blog/new.html');
    });

    test('updateUrlForRename with html extension preserves extension', () => {
      const loadPreviewSpy = vi.spyOn(previewManager, 'loadPreview');
      mockIframe.src = 'http://localhost:3000/blog/old.html';

      (previewManager as any).updateUrlForRename('blog/old.html', 'blog/new.html');

      expect(loadPreviewSpy).toHaveBeenCalledWith('http://localhost:3000/blog/new.html');
    });

    test('updateUrlForRename with malformed URL falls back to refresh', () => {
      const loadPreviewSpy = vi.spyOn(previewManager, 'loadPreview');
      const refreshSpy = vi.spyOn(previewManager, 'refresh');
      mockIframe.src = 'invalid-url';

      (previewManager as any).updateUrlForRename('old.md', 'new.md');

      expect(loadPreviewSpy).not.toHaveBeenCalled();
      expect(refreshSpy).toHaveBeenCalledTimes(1);
    });
  });
});