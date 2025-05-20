import type { CreateClientConfig } from './client/client.gen';
import { SERVICE_URL } from '$env/static/private';

export const createClientConfig: CreateClientConfig = (config) => ({
  ...config,
  baseUrl: SERVICE_URL,
});
