import { build } from "esbuild";
import { readdir, stat, unlink } from "node:fs/promises";
import { createRequire } from "node:module";
import { dirname, resolve } from "node:path";

const skinRequire = createRequire(import.meta.resolve("skinview3d"));
const threeModule = resolve(
  dirname(skinRequire.resolve("three")),
  "three.module.js",
);

const result = await build({
  entryPoints: {
    cinematic: "website/visuals/motion.js",
    studio: "website/visuals/studio.js",
  },
  outdir: "website",
  splitting: true,
  chunkNames: "assets/visuals/[name]-[hash]",
  bundle: true,
  minify: true,
  format: "esm",
  target: ["es2022"],
  plugins: [
    {
      name: "shared-three",
      setup(builder) {
        builder.onResolve({ filter: /^three$/ }, () => ({ path: threeModule }));
      },
    },
  ],
  metafile: true,
  legalComments: "eof",
});
const chunkDirectory = resolve("website/assets/visuals");
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
