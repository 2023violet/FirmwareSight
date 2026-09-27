import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';

import { App } from './App';
import './styles/tokens.css';
import './styles/global.css';

const container = document.getElementById('root');
if (container === null) {
  throw new Error('The FirmwareSight root container is missing from index.html.');
}

createRoot(container).render(
  <StrictMode>
    <App />
  </StrictMode>,
);
