import js from '@eslint/js';
import tseslint from 'typescript-eslint';

export default tseslint.config(
  {
    // `src/ipc/generated` is ts-rs output and is never hand-edited, so linting it would only
    // produce findings nobody is allowed to fix.
    ignores: ['dist/**', 'node_modules/**', 'src/ipc/generated/**', 'coverage/**'],
  },
  js.configs.recommended,
  tseslint.configs.recommended,
  {
    files: ['**/*.{ts,tsx}'],
    rules: {
      '@typescript-eslint/consistent-type-imports': [
        'error',
        { prefer: 'type-imports', fixStyle: 'separate-type-imports' },
      ],
      '@typescript-eslint/no-floating-promises': 'error',
      '@typescript-eslint/no-misused-promises': 'error',
    },
    languageOptions: {
      parserOptions: {
        // Type-aware linting: `await` mistakes and unhandled promises are exactly the class of
        // bug a thin shell is prone to, and they are only visible with type information.
        projectService: true,
        tsconfigRootDir: import.meta.dirname,
      },
    },
  },
  {
    files: ['**/*.test.{ts,tsx}', 'src/test/**'],
    rules: {
      '@typescript-eslint/no-unused-expressions': 'off',
    },
  },
);
