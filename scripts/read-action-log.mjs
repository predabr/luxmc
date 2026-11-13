import { githubToken } from './github-authenticated.mjs';
import { writeFileSync } from 'node:fs';
const response = await fetch(`https://api.github.com/repos/predabr/luxmc/actions/jobs/${process.argv[2]}/logs`, { headers: { Authorization: `Bearer ${githubToken()}`, Accept: 'application/vnd.github+json' }, signal: AbortSignal.timeout(60000) });
if (!response.ok) throw new Error(`Log indisponível: ${response.status}`);
const text = await response.text();
if (process.argv[3]) writeFileSync(process.argv[3], text);
console.log(text.split('\n').slice(-100).join('\n'));
