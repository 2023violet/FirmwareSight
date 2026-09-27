import { cleanup } from '@testing-library/react';
import { afterEach } from 'vitest';

// RTL's automatic cleanup needs a global `afterEach`; this project runs Vitest without globals,
// so the teardown is registered explicitly instead.
afterEach(() => {
  cleanup();
});
