const prettier = ['prettier --write'];

export default {
    'src/**/*.{js,jsx,ts,tsx}': ['eslint --fix', ...prettier],
    'src/**/*.{json,css,scss,md,mdx,html}': prettier,
    'bindings/**/*.ts': prettier,
    'package.json': prettier,
    'tsconfig.json': prettier,
    'vite.config.ts': prettier,

    'src/**/*.rs': ['rustfmt --edition 2024', () => 'sleep 2'],
};
