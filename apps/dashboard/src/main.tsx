/**
 * @license
 * SPDX-License-Identifier: Apache-2.0
 */

import React from 'react';
import ReactDOM from 'react-dom/client';
import App from './App';
import './index.css';

// Environment validation - ensure required VITE_* variables are present
const REQUIRED_ENV_VARS = [
  'VITE_API_BASE',
  'VITE_ENGINE_MODE',
];

const missing = REQUIRED_ENV_VARS.filter(key => !import.meta.env[key]);
if (missing.length > 0) {
  throw new Error(`Missing required environment variables: ${missing.join(', ')}`);
}

console.log('✅ Environment validation passed');

ReactDOM.createRoot(document.getElementById('root') as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>
);
