const production = !process.env.ROLLUP_WATCH;
module.exports = {
  future: {
    purgeLayersByDefault: true,
    removeDeprecatedGapUtilities: true,
  },
  plugins: [
  ],
  content: [
    "./brr.css",
    "./src/App.svelte",
    "./src/index.css",
    { raw: `@tailwind base;
  @tailwind components;
  @tailwind utilities;
`, extension: 'css' }
  ],
};
