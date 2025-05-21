import { SERVICE_URL } from '$env/static/private';
import type { CreateClientConfig } from './requests/client.gen';

export const createClientConfig: CreateClientConfig = (config) => ({
	...config,
	baseUrl: SERVICE_URL
});
