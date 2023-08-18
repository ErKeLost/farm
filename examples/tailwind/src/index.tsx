import React from 'react';
import { createRoot } from 'react-dom/client';
import App from './App';
import './index.css';
import pkg from '../package.json';
import a from '../aa.jsonc';
console.log(pkg);
console.log(a);

const container = document.querySelector('#root');
const root = createRoot(container);

root.render(<App />);
