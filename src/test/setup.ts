import '@testing-library/jest-dom/vitest';

// jsdom does not implement ResizeObserver; stub it so components that use it
// can mount without throwing.
window.ResizeObserver = class ResizeObserver {
    observe() {}
    unobserve() {}
    disconnect() {}
};
