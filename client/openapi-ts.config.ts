import { defaultPlugins } from '@hey-api/openapi-ts';

export default {
	input: 'https://eerip.onrender.com/api-docs/openapi.json',
	output: 'src/lib/server/requests',
	plugins: [
		...defaultPlugins,
		'@hey-api/client-fetch',
		'@tanstack/svelte-query',
		'zod',
		{
			name: '@hey-api/client-fetch',
			runtimeConfigPath: './src/lib/server/client.ts'
		}
	]
};
