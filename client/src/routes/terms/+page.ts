import { error } from '@sveltejs/kit';
import type { PageLoad } from './$types';

export const load: PageLoad = ({ params, url }) => {
	return {
		email: `info@${url.origin}`,
		jurisdiction: 'Wakanda'
	};
};
