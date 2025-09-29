// Backend Integration Unit Tests
// Tests Tauri command wrappers and configuration utilities

import { describe, test, expect, beforeEach, afterEach, vi } from 'vitest';
import { getCompileConfig, compileWithProgress, initializeTauriBackend } from './tauri-integration';
import { Channel } from '@tauri-apps/api/core';

// Mock Tauri API
vi.mock('@tauri-apps/api/core', () => ({
  Channel: vi.fn().mockImplementation(() => ({
    onmessage: null,
  })),
}));

// Mock the bindings
vi.mock('../bindings', () => ({
  commands: {
    compileFolder: vi.fn(),
  },
}));

import { commands } from '../bindings';

describe('Backend Integration', () => {
  beforeEach(() => {
    // Reset window object
    delete (window as any).__MOSS_COMPILE_CONFIG__;

    // Set up window.tauriInvoke with mocked commands
    (window as any).tauriInvoke = commands;

    vi.clearAllMocks();
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  describe('getCompileConfig', () => {
    test('returns config when window.__MOSS_COMPILE_CONFIG__ exists', () => {
      const mockConfig = {
        folder_path: '/Users/test/Documents/my-site',
        auto_serve: true,
      };

      (window as any).__MOSS_COMPILE_CONFIG__ = mockConfig;

      const result = getCompileConfig();
      expect(result).toEqual(mockConfig);
    });

    test('returns null when window.__MOSS_COMPILE_CONFIG__ is undefined', () => {
      const result = getCompileConfig();
      expect(result).toBeNull();
    });

    test('returns null when window.__MOSS_COMPILE_CONFIG__ is null', () => {
      (window as any).__MOSS_COMPILE_CONFIG__ = null;

      const result = getCompileConfig();
      expect(result).toBeNull();
    });

    test('handles config with special characters in folder path', () => {
      const mockConfig = {
        folder_path: '/Users/test/My Documents/Site with spaces & symbols!',
        auto_serve: false,
      };

      (window as any).__MOSS_COMPILE_CONFIG__ = mockConfig;

      const result = getCompileConfig();
      expect(result).toEqual(mockConfig);
    });

    test('handles config with auto_serve false', () => {
      const mockConfig = {
        folder_path: '/test/path',
        auto_serve: false,
      };

      (window as any).__MOSS_COMPILE_CONFIG__ = mockConfig;

      const result = getCompileConfig();
      expect(result).toEqual(mockConfig);
      expect(result?.auto_serve).toBe(false);
    });
  });

  describe('initializeTauriBackend', () => {
    test('sets window.tauriInvoke to commands', async () => {
      await initializeTauriBackend();

      expect(window.tauriInvoke).toBe(commands);
    });

    test('can be called multiple times safely', async () => {
      await initializeTauriBackend();
      await initializeTauriBackend();

      expect(window.tauriInvoke).toBe(commands);
    });
  });

  describe('compileWithProgress', () => {
    let mockProgressCallback: ReturnType<typeof vi.fn>;
    let mockChannel: any;

    beforeEach(() => {
      mockProgressCallback = vi.fn();
      mockChannel = {
        onmessage: null,
      };
      (Channel as any).mockReturnValue(mockChannel);
    });

    test('creates progress channel and sets up listener', async () => {
      const mockResult = { status: 'ok', data: 'Success message' };
      (commands.compileFolder as any).mockResolvedValue(mockResult);

      await compileWithProgress('/test/folder', true, false, mockProgressCallback);

      expect(Channel).toHaveBeenCalled();
      expect(mockChannel.onmessage).toBe(mockProgressCallback);
    });

    test('calls compileFolder with correct parameters', async () => {
      const mockResult = { status: 'ok', data: 'Success message' };
      (commands.compileFolder as any).mockResolvedValue(mockResult);

      await compileWithProgress('/test/folder', true, false, mockProgressCallback);

      expect(commands.compileFolder).toHaveBeenCalledWith(
        '/test/folder',
        true,
        false,
        mockChannel
      );
    });

    test('returns success message when compilation succeeds', async () => {
      const mockResult = { status: 'ok', data: 'Compilation successful!' };
      (commands.compileFolder as any).mockResolvedValue(mockResult);

      const result = await compileWithProgress('/test/folder', true, false, mockProgressCallback);

      expect(result).toBe('Compilation successful!');
    });

    test('throws error when compilation fails', async () => {
      const mockResult = { status: 'error', error: 'Folder not found' };
      (commands.compileFolder as any).mockResolvedValue(mockResult);

      await expect(
        compileWithProgress('/nonexistent/folder', true, mockProgressCallback)
      ).rejects.toThrow('Folder not found');
    });

    test('uses default autoServe value when not provided', async () => {
      const mockResult = { status: 'ok', data: 'Success' };
      (commands.compileFolder as any).mockResolvedValue(mockResult);

      await compileWithProgress('/test/folder', undefined, false, mockProgressCallback);

      expect(commands.compileFolder).toHaveBeenCalledWith(
        '/test/folder',
        true, // default value
        false, // default watch value
        mockChannel
      );
    });

    test('handles autoServe false explicitly', async () => {
      const mockResult = { status: 'ok', data: 'Success' };
      (commands.compileFolder as any).mockResolvedValue(mockResult);

      await compileWithProgress('/test/folder', false, false, mockProgressCallback);

      expect(commands.compileFolder).toHaveBeenCalledWith(
        '/test/folder',
        false,
        false, // default watch value
        mockChannel
      );
    });

    test('propagates Tauri command errors', async () => {
      const tauriError = new Error('Tauri connection failed');
      (commands.compileFolder as any).mockRejectedValue(tauriError);

      await expect(
        compileWithProgress('/test/folder', true, mockProgressCallback)
      ).rejects.toThrow('Tauri connection failed');
    });

    test('handles progress callback with sample progress update', async () => {
      const mockResult = { status: 'ok', data: 'Success' };
      (commands.compileFolder as any).mockResolvedValue(mockResult);

      await compileWithProgress('/test/folder', true, false, mockProgressCallback);

      // Simulate progress update
      const sampleUpdate = {
        step: 'scanning',
        message: 'Scanning files...',
        percentage: 25,
        completed: false,
        port: null,
      };

      mockChannel.onmessage(sampleUpdate);

      expect(mockProgressCallback).toHaveBeenCalledWith(sampleUpdate);
    });

    test('handles path with special characters', async () => {
      const mockResult = { status: 'ok', data: 'Success' };
      (commands.compileFolder as any).mockResolvedValue(mockResult);
      const specialPath = '/Users/test/Folder with spaces & symbols!';

      await compileWithProgress(specialPath, true, false, mockProgressCallback);

      expect(commands.compileFolder).toHaveBeenCalledWith(
        specialPath,
        true,
        false, // default watch value
        mockChannel
      );
    });
  });

  describe('Type Safety', () => {
    test('getCompileConfig returns properly typed config', () => {
      const config = {
        folder_path: '/test/path',
        auto_serve: true,
      };

      (window as any).__MOSS_COMPILE_CONFIG__ = config;

      const result = getCompileConfig();

      // TypeScript should enforce these types
      if (result) {
        expect(typeof result.folder_path).toBe('string');
        expect(typeof result.auto_serve).toBe('boolean');
      }
    });

    test('compileWithProgress accepts proper callback signature', async () => {
      const mockResult = { status: 'ok', data: 'Success' };
      (commands.compileFolder as any).mockResolvedValue(mockResult);

      // This should compile without TypeScript errors
      const callback = (update: { step: string; message: string; percentage: number; completed: boolean; port: number | null }) => {
        console.log(`Step: ${update.step}, Progress: ${update.percentage}%`);
      };

      await compileWithProgress('/test/folder', true, false, callback);

      expect(commands.compileFolder).toHaveBeenCalled();
    });
  });

  describe('Edge Cases', () => {
    test('handles empty folder path', async () => {
      const mockResult = { status: 'error', error: 'Empty folder path' };
      (commands.compileFolder as any).mockResolvedValue(mockResult);

      await expect(
        compileWithProgress('', true, vi.fn())
      ).rejects.toThrow('Empty folder path');
    });

    test('handles malformed config object', () => {
      // Malformed config missing required fields
      (window as any).__MOSS_COMPILE_CONFIG__ = { invalid: 'config' };

      const result = getCompileConfig();

      // Should still return the object (validation happens elsewhere)
      expect(result).toEqual({ invalid: 'config' });
    });

    test('handles config with extra fields', () => {
      const configWithExtraFields = {
        folder_path: '/test/path',
        auto_serve: true,
        extra_field: 'should be ignored',
        another_field: 123,
      };

      (window as any).__MOSS_COMPILE_CONFIG__ = configWithExtraFields;

      const result = getCompileConfig();
      expect(result).toEqual(configWithExtraFields);
    });
  });
});