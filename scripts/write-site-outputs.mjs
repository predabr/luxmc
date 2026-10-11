import { mkdir, readFile, rename, unlink, writeFile } from 'node:fs/promises';
import { randomUUID } from 'node:crypto';
import { dirname, resolve, sep } from 'node:path';

export async function writeSiteOutputs(files, directory = 'website') {
    const root = resolve(directory);
    for (const file of files) {
        const path = resolve(file.path);
        if (!path.startsWith(root + sep)) throw new Error('Build output outside the website directory');
        const existing = await readFile(path).catch(error => {
            if (error.code === 'ENOENT') return undefined;
            throw error;
        });
        if (existing?.equals(file.contents)) continue;
        await mkdir(dirname(path), { recursive: true });
        const temporary = `${path}.${randomUUID()}.tmp`;
        try {
            await writeFile(temporary, file.contents, { flag: 'wx' });
            await rename(temporary, path);
        } finally {
            await unlink(temporary).catch(error => {
                if (error.code !== 'ENOENT') throw error;
            });
        }
    }
}
