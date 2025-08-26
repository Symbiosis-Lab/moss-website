// Preview window control bar functionality
// Handles communication between preview UI and Tauri backend

import { invoke } from '@tauri-apps/api/core';

class PreviewControlBar {
    constructor() {
        this.previewId = null;
        this.state = {
            isPublished: false,
            isPublishing: false,
            syndicationTargets: [],
            previewUrl: null
        };
        
        this.elements = {
            publishBtn: document.getElementById('publish-btn'),
            editBtn: document.getElementById('edit-btn'),
            syndicateBtn: document.getElementById('syndicate-btn'),
            closeBtn: document.getElementById('close-btn'),
            statusDot: document.getElementById('status-dot'),
            statusText: document.getElementById('status-text'),
            previewTitle: document.getElementById('preview-title'),
            syndicationTags: document.getElementById('syndication-tags'),
            previewIframe: document.getElementById('preview-iframe'),
            loading: document.getElementById('loading')
        };
        
        this.init();
    }
    
    async init() {
        // Get preview ID from URL parameters
        const urlParams = new URLSearchParams(window.location.search);
        this.previewId = urlParams.get('preview_id');
        
        if (!this.previewId) {
            console.error('No preview ID provided');
            this.showError('Invalid preview session');
            return;
        }
        
        // Set up event listeners
        this.setupEventListeners();
        
        // Load initial state
        await this.loadPreviewState();
        
        // Load preview content
        this.loadPreviewContent();
    }
    
    setupEventListeners() {
        this.elements.publishBtn.addEventListener('click', () => this.handlePublish());
        this.elements.editBtn.addEventListener('click', () => this.handleEdit());
        this.elements.syndicateBtn.addEventListener('click', () => this.handleSyndicate());
        this.elements.closeBtn.addEventListener('click', () => this.handleClose());
        
        // Handle iframe load event
        this.elements.previewIframe.addEventListener('load', () => {
            this.elements.loading.style.display = 'none';
            this.elements.previewIframe.style.display = 'block';
        });
        
        this.elements.previewIframe.addEventListener('error', () => {
            this.elements.loading.textContent = 'Failed to load preview';
        });
    }
    
    async loadPreviewState() {
        try {
            const state = await invoke('get_preview_state', {
                previewId: this.previewId
            });
            
            this.updateState({
                isPublished: state.is_published,
                syndicationTargets: state.syndication_targets || [],
                previewUrl: state.url
            });
            
            // Update title with folder name
            const folderName = state.folder_path.split('/').pop() || 'Preview';
            this.elements.previewTitle.textContent = `${folderName} - moss Preview`;
            
        } catch (error) {
            console.error('Failed to load preview state:', error);
            this.showError('Failed to load preview state');
        }
    }
    
    loadPreviewContent() {
        if (this.state.previewUrl) {
            this.elements.previewIframe.src = this.state.previewUrl;
        }
    }
    
    async handlePublish() {
        if (this.state.isPublishing || this.state.isPublished) {
            return;
        }
        
        this.updateState({ isPublishing: true });
        
        try {
            const result = await invoke('publish_from_preview', {
                previewId: this.previewId,
                platform: 'moss.pub'
            });
            
            console.log('Published successfully:', result);
            
            // Refresh state
            await this.loadPreviewState();
            
            // Show success message
            this.showSuccess('Published successfully!');
            
        } catch (error) {
            console.error('Publish failed:', error);
            this.showError(`Publish failed: ${error}`);
        } finally {
            this.updateState({ isPublishing: false });
        }
    }
    
    async handleEdit() {
        try {
            const result = await invoke('open_editor_from_preview', {
                previewId: this.previewId
            });
            
            console.log('Opened editor:', result);
            this.showSuccess('Opened folder for editing');
            
        } catch (error) {
            console.error('Failed to open editor:', error);
            this.showError(`Failed to open editor: ${error}`);
        }
    }
    
    async handleSyndicate() {
        // Simple syndication target input
        const target = prompt('Enter syndication target (e.g., twitter, dev.to):');
        
        if (!target || !target.trim()) {
            return;
        }
        
        try {
            const result = await invoke('add_syndication_target', {
                previewId: this.previewId,
                target: target.trim()
            });
            
            console.log('Added syndication target:', result);
            
            // Refresh state
            await this.loadPreviewState();
            
            this.showSuccess(`Added syndication target: ${target}`);
            
        } catch (error) {
            console.error('Failed to add syndication target:', error);
            this.showError(`Failed to add syndication: ${error}`);
        }
    }
    
    async handleClose() {
        try {
            await invoke('close_preview_window_cmd', {
                previewId: this.previewId
            });
        } catch (error) {
            console.error('Failed to close window:', error);
        }
    }
    
    updateState(newState) {
        this.state = { ...this.state, ...newState };
        this.updateUI();
    }
    
    updateUI() {
        // Update publish button
        const publishEnabled = !this.state.isPublished && !this.state.isPublishing;
        this.elements.publishBtn.disabled = !publishEnabled;
        
        if (this.state.isPublishing) {
            this.elements.publishBtn.innerHTML = '<span>⏳</span> Publishing...';
        } else if (this.state.isPublished) {
            this.elements.publishBtn.innerHTML = '<span>✅</span> Published';
        } else {
            this.elements.publishBtn.innerHTML = '<span>📤</span> Publish';
        }
        
        // Update status indicator
        if (this.state.isPublished) {
            this.elements.statusDot.className = 'status-dot';
            this.elements.statusText.textContent = 'Published';
        } else {
            this.elements.statusDot.className = 'status-dot unpublished';
            this.elements.statusText.textContent = 'Ready to publish';
        }
        
        // Update syndication tags
        this.elements.syndicationTags.innerHTML = '';
        this.state.syndicationTargets.forEach(target => {
            const tag = document.createElement('div');
            tag.className = 'tag';
            tag.textContent = target;
            this.elements.syndicationTags.appendChild(tag);
        });
    }
    
    showSuccess(message) {
        // Simple success feedback - could be enhanced with a toast system
        const originalText = this.elements.statusText.textContent;
        this.elements.statusText.textContent = message;
        this.elements.statusText.style.color = '#34C759';
        
        setTimeout(() => {
            this.elements.statusText.textContent = originalText;
            this.elements.statusText.style.color = '';
        }, 3000);
    }
    
    showError(message) {
        console.error(message);
        const originalText = this.elements.statusText.textContent;
        this.elements.statusText.textContent = message;
        this.elements.statusText.style.color = '#FF3B30';
        
        setTimeout(() => {
            this.elements.statusText.textContent = originalText;
            this.elements.statusText.style.color = '';
        }, 5000);
    }
}

// Initialize when DOM is loaded
document.addEventListener('DOMContentLoaded', () => {
    new PreviewControlBar();
});

// Validation functions for testing
export function validatePublishState(state) {
    return !state.isPublished && !state.isPublishing;
}

export function validateSyndicationTarget(target) {
    if (!target || typeof target !== 'string') {
        return false;
    }
    
    const trimmed = target.trim();
    if (trimmed.length === 0) {
        return false;
    }
    
    // Only allow alphanumeric, dots, hyphens, underscores
    const validPattern = /^[a-zA-Z0-9.\-_]+$/;
    return validPattern.test(trimmed);
}

export function getEnabledActions(state) {
    const actions = [];
    
    if (validatePublishState(state)) {
        actions.push('publish');
    }
    
    actions.push('edit');
    actions.push('syndicate');
    actions.push('close');
    
    return actions;
}