// Dark mode toggle functionality
function toggleTheme() {
    const html = document.documentElement;
    const currentTheme = html.getAttribute('data-theme');
    const newTheme = currentTheme === 'dark' ? 'light' : 'dark';
    
    html.setAttribute('data-theme', newTheme);
    localStorage.setItem('theme', newTheme);
    console.log('🎨 Theme manually toggled to:', newTheme);
}

// Initialize theme system with robust dark mode detection
// References: 
// - MDN: https://developer.mozilla.org/en-US/docs/Web/CSS/@media/prefers-color-scheme
// - Specification: Media Queries Level 5 - prefers-color-scheme
// - Browser Support: Widely available since January 2020 (Chrome 76+, Firefox 67+, Safari 12.1+)
(function() {
    console.log('🎨 Initializing theme system...');
    
    // Feature detection for matchMedia support
    const supportsMatchMedia = window.matchMedia && window.matchMedia('(prefers-color-scheme)').media !== 'not all';
    console.log('🔍 matchMedia support:', supportsMatchMedia);
    
    let prefersDark = false;
    
    if (supportsMatchMedia) {
        try {
            const darkModeQuery = window.matchMedia('(prefers-color-scheme: dark)');
            prefersDark = darkModeQuery.matches;
            console.log('🌙 System prefers dark mode:', prefersDark);
            
            // Listen for real-time system theme changes
            // Using modern addEventListener (addListener is deprecated)
            darkModeQuery.addEventListener('change', function(event) {
                const savedTheme = localStorage.getItem('theme');
                // Only auto-switch if user hasn't manually set a preference
                if (!savedTheme) {
                    const newTheme = event.matches ? 'dark' : 'light';
                    console.log('🔄 System theme changed to:', newTheme);
                    document.documentElement.setAttribute('data-theme', newTheme);
                } else {
                    console.log('⚙️ System theme changed but user has manual preference:', savedTheme);
                }
            });
        } catch (error) {
            console.warn('⚠️ Error detecting dark mode preference:', error);
            prefersDark = false; // Fallback to light mode
        }
    } else {
        console.warn('⚠️ matchMedia not supported, falling back to light mode');
    }
    
    // Determine initial theme: user preference > system preference > light default
    const savedTheme = localStorage.getItem('theme');
    const initialTheme = savedTheme || (prefersDark ? 'dark' : 'light');
    
    console.log('🎯 Theme decision - saved:', savedTheme, 'system prefers dark:', prefersDark, 'final:', initialTheme);
    
    // Apply the theme
    document.documentElement.setAttribute('data-theme', initialTheme);
    console.log('✅ Theme system initialized with:', initialTheme);
})();