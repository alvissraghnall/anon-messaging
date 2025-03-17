import type { PageLoad } from "./$types";

export const load: PageLoad = ({ url }) => {
    return {
        email: `info@${url.hostname}`,
    }
}

export const ssr = true