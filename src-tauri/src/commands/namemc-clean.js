(() => {
    if (location.protocol !== 'https:' || !(location.hostname === 'namemc.com' || location.hostname.endsWith('.namemc.com'))) return;
    const apply = () => {
        if (document.getElementById('luxmc-namemc-clean')) return;
        const style = document.createElement('style');
        style.id = 'luxmc-namemc-clean';
        style.textContent = `ins.adsbygoogle, .ad-container, .ad-slot, .ad-placeholder, [data-ad-slot], [data-ad-unit-id], [id^="adngin-"], iframe[src*="doubleclick.net/"], iframe[src*="googlesyndication.com/"], iframe[src*="amazon-adsystem.com/"] { display: none !important; min-height: 0 !important; height: 0 !important; margin: 0 !important; padding: 0 !important; }`;
        (document.head || document.documentElement).appendChild(style);
    };
    if (document.documentElement) apply();
    else document.addEventListener('DOMContentLoaded', apply, { once: true });
})();
