import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { materialThemeBuilderTailwindPlugin } from '@nexim/tailwind-material-colors';

const projectRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');
const generatedThemeCssPath = path.join(projectRoot, 'target', 'material-theme.css');
const fallbackSourceColor = '#536b73';

let sourceColor = fallbackSourceColor;
if (fs.existsSync(generatedThemeCssPath)) {
  const generatedCss = fs.readFileSync(generatedThemeCssPath, 'utf8');
  const match = generatedCss.match(/--material-source-color:\s*(#[0-9a-f]{6})\s*;/i);
  if (!match) {
    throw new Error('[material-theme] generated CSS does not contain a valid --material-source-color.');
  }
  sourceColor = match[1];
}

export default materialThemeBuilderTailwindPlugin({
  primaryColor: sourceColor,
  scheme: 'tonalSpot',
});
