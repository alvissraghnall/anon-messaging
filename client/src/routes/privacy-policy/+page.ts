import type { PageLoad } from "./$types";

export const load: PageLoad = ({ url }) => {
    return {
        email: `info@${url.origin}`,
		pageUrl: url.origin
    }
}

export const ssr = true
