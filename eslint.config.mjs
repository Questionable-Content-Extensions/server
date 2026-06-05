import js from '@eslint/js';
import reactHooks from 'eslint-plugin-react-hooks';
import globals from 'globals';
import tseslint from 'typescript-eslint';

export default tseslint.config(
    { ignores: ['build/**', 'generated/**', 'public/**'] },

    js.configs.recommended,

    ...tseslint.configs.recommended,

    reactHooks.configs['flat']['recommended-latest'],

    {
        languageOptions: {
            globals: globals.browser,
        },
    },

    {
        files: ['commitlint.config.js', 'vite.config.ts'],
        languageOptions: {
            globals: globals.node,
        },
    },
);
