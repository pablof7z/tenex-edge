import { cp, mkdir, readFile, rm, writeFile } from 'node:fs/promises';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const siteDir = dirname(fileURLToPath(import.meta.url));
const repoRoot = dirname(siteDir);
const output = join(siteDir, 'dist');
const skillSource = join(repoRoot, 'skills', 'mosaico-setup', 'SKILL.md');

await rm(output, { recursive: true, force: true });
await mkdir(output, { recursive: true });

for (const file of ['index.html', 'styles.css', 'site.js']) {
  await cp(join(siteDir, file), join(output, file));
}

const setup = await readFile(skillSource, 'utf8');
if (!setup.startsWith('---\nname: mosaico-setup\n')) {
  throw new Error('skills/mosaico-setup/SKILL.md is not the canonical setup skill');
}
await writeFile(join(output, 'SETUP.md'), setup);

console.log(`Built ${output}`);
