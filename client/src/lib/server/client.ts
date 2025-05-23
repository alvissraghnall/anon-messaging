import { SERVICE_URL } from '$env/static/private';
import { createClient, type Client } from '@hey-api/client-fetch';

export const generalClient: Client = createClient({
	baseUrl: SERVICE_URL,
	headers: {
		'content-type': 'application/json'
	},
	mode: 'no-cors'
});
