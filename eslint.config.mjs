import js from '@eslint/js';
import reactHooks from 'eslint-plugin-react-hooks';
import globals from 'globals';

export default [
    { ignores: ['build/**', 'generated/**', 'public/**'] },

    js.configs.recommended,

    reactHooks.configs['flat']['recommended-latest'],

    {
        languageOptions: {
            globals: globals.browser,
        },
    },

    {
        files: ['commitlint.config.js', 'vite.config.js'],
        languageOptions: {
            globals: globals.node,
        },
    },
];
