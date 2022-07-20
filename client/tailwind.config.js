const production = !process.env.ROLLUP_WATCH;
module.exports = {
  future: {
    purgeLayersByDefault: true,
    removeDeprecatedGapUtilities: true,
  },
  plugins: [
  ],
  purge: {
    content: [
     "./src/App.svelte",
     "./src/index.css"
    ],
    enabled: production // disable purge in dev
  },
  content: [
       "./src/App.svelte",
       "./src/index.css"
  ],
};
