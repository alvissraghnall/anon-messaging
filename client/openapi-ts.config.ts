import { defaultPlugins } from '@hey-api/openapi-ts';

export default {
  input: './api.json',
  output: 'src/lib/requests',
  plugins: [
    ...defaultPlugins,
    '@hey-api/client-fetch',
    '@tanstack/svelte-query', 
  ],
};
