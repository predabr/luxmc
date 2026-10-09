import { execFileSync } from 'node:child_process';
import { readFileSync, writeFileSync } from 'node:fs';

export function githubToken() {
    if (process.env.GH_TOKEN || process.env.GITHUB_TOKEN) return process.env.GH_TOKEN || process.env.GITHUB_TOKEN;
    const credential = execFileSync('git', ['credential', 'fill'], { input: 'protocol=https\nhost=github.com\n\n', encoding: 'utf8', windowsHide: true, env: { ...process.env, GIT_TERMINAL_PROMPT: '0', GCM_INTERACTIVE: 'never' } });
    const token = credential.split('\n').find(line => line.startsWith('password='))?.slice(9).trim();
    if (!token) throw new Error('Credencial GitHub indisponível.');
    return token;
}
export async function githubRequest(path, { method = 'GET', body } = {}) {
    const response = await fetch(`https://api.github.com/${path}`, { method, headers: { Authorization: `Bearer ${githubToken()}`, Accept: 'application/vnd.github+json', 'X-GitHub-Api-Version': '2022-11-28', 'Content-Type': 'application/json' }, body: body ? JSON.stringify(body) : undefined, signal: AbortSignal.timeout(60000) });
    if (!response.ok) throw new Error(`GitHub API ${response.status} em ${path}`);
    return response.status === 204 ? null : response.json();
}
if (process.argv[1]?.replaceAll('\\', '/').endsWith('/github-authenticated.mjs')) {
    const [path = 'repos/predabr/luxmc', output, bodyFile] = process.argv.slice(2);
    const result = await githubRequest(path, bodyFile ? { method: 'PATCH', body: JSON.parse(readFileSync(bodyFile, 'utf8')) } : {});
    if (output) writeFileSync(output, JSON.stringify(result, null, 2) + '\n');
    else console.log(JSON.stringify({ access: true, private: result?.private, defaultBranch: result?.default_branch, canPush: result?.permissions?.push }));
}
