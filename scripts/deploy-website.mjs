import { spawnSync } from 'node:child_process';
import { accessSync } from 'node:fs';
import { fileURLToPath } from 'node:url';

const directory = fileURLToPath(new URL('../website/', import.meta.url));
for (const path of ['wrangler.toml', 'functions/_middleware.js', 'functions/api/social/[action].js', 'functions/api/commerce/[action].js']) accessSync(fileURLToPath(new URL(`../website/${path}`, import.meta.url)));
const argumentsList = ['pages', 'deploy', '.', '--project-name=luxmc', '--branch=main', '--commit-dirty=true'];
const wrangler = process.env.LUXMC_WRANGLER_BINARY;
const pnpm = process.env.npm_execpath;
if (!wrangler && (!pnpm || !/pnpm/i.test(pnpm))) throw new Error('Execute a publicação com pnpm deploy:website.');
const result = spawnSync(process.execPath, wrangler ? [wrangler, ...argumentsList] : [pnpm, 'dlx', 'wrangler@4.135.0', ...argumentsList], { cwd: directory, env: process.env, stdio: 'inherit', windowsHide: true });
if (result.error) throw result.error;
process.exitCode = result.status ?? 1;
