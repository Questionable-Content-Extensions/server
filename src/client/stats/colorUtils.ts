function hexToRgb(hex: string): [number, number, number] {
    return [
        parseInt(hex.slice(0, 2), 16) / 255,
        parseInt(hex.slice(2, 4), 16) / 255,
        parseInt(hex.slice(4, 6), 16) / 255,
    ];
}

function rgbToHsl(r: number, g: number, b: number): [number, number, number] {
    const max = Math.max(r, g, b);
    const min = Math.min(r, g, b);
    const l = (max + min) / 2;
    if (max === min) return [0, 0, l];
    const d = max - min;
    const s = l > 0.5 ? d / (2 - max - min) : d / (max + min);
    let h: number;
    if (max === r) h = ((g - b) / d + (g < b ? 6 : 0)) / 6;
    else if (max === g) h = ((b - r) / d + 2) / 6;
    else h = ((r - g) / d + 4) / 6;
    return [h, s, l];
}

function hslToRgbHex(h: number, s: number, l: number): string {
    const c = (1 - Math.abs(2 * l - 1)) * s;
    const x = c * (1 - Math.abs(((h * 6) % 2) - 1));
    const m = l - c / 2;
    let r: number, g: number, b: number;
    const hi = Math.floor(h * 6) % 6;
    if (hi === 0) [r, g, b] = [c, x, 0];
    else if (hi === 1) [r, g, b] = [x, c, 0];
    else if (hi === 2) [r, g, b] = [0, c, x];
    else if (hi === 3) [r, g, b] = [0, x, c];
    else if (hi === 4) [r, g, b] = [x, 0, c];
    else [r, g, b] = [c, 0, x];
    const hex = (v: number) =>
        Math.round(Math.max(0, Math.min(1, v + m)) * 255)
            .toString(16)
            .padStart(2, '0');
    return `#${hex(r)}${hex(g)}${hex(b)}`;
}

// Light mode: clamp lightness to [0.28, 0.58] — dark enough to see on white,
// light enough to see on a white-ish surface.
// Dark mode: clamp to [0.42, 0.82] — bright enough to see on dark backgrounds.
const LIGHT_L_MIN = 0.28;
const LIGHT_L_MAX = 0.58;
const DARK_L_MIN = 0.42;
const DARK_L_MAX = 0.82;

/**
 * Returns a CSS color string for `hex` (6-char, no `#`) that is guaranteed to
 * contrast against the background. Hue and saturation are preserved; only
 * lightness is clamped.
 */
export function contrastColor(hex: string, dark = false): string {
    const [r, g, b] = hexToRgb(hex);
    const [h, s, l] = rgbToHsl(r, g, b);
    const [lMin, lMax] = dark
        ? [DARK_L_MIN, DARK_L_MAX]
        : [LIGHT_L_MIN, LIGHT_L_MAX];
    const clamped = Math.max(lMin, Math.min(lMax, l));
    if (clamped === l) return `#${hex}`;
    return hslToRgbHex(h, s, clamped);
}
