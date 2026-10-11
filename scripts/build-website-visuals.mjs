import { build } from "esbuild";
import { mkdir, readdir, stat, unlink } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { writeSiteOutputs } from "./write-site-outputs.mjs";


const result = await build({
  entryPoints: {
    cinematic: "site-client/visuals/motion.ts",
  },
  outdir: "website",
  splitting: true,
  chunkNames: "assets/visuals/[name]-[hash]",
  bundle: true,
  minify: true,
  format: "esm",
  target: ["es2022"],
  metafile: true,
  legalComments: "eof",
  write: false,
});
await writeSiteOutputs(result.outputFiles);
const chunkDirectory = resolve("website/assets/visuals");
await mkdir(chunkDirectory, { recursive: true });
const outputs = new Set(
  Object.keys(result.metafile.outputs).map((path) => resolve(path)),
);
for (const filename of await readdir(chunkDirectory)) {
  const path = resolve(chunkDirectory, filename);
  if (
    dirname(path) === chunkDirectory &&
    filename.endsWith(".js") &&
    !outputs.has(path)
  )
    await unlink(path);
}
console.log(
  `Website visuals built: ${Math.round((await stat("website/cinematic.js")).size / 1024)} KiB`,
);
