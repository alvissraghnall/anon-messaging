import { db, UserRepository } from '$lib/server/db';
import type { PageServerLoad, Actions } from './$types';
import { fail } from '@sveltejs/kit';
import { SERVICE_URL } from '$env/static/private';
import { createClient } from '@hey-api/client-fetch';
import { registerUserHandler } from '$lib/server/requests';

let client = createClient({
	baseUrl: SERVICE_URL,
})

let userRepo = new UserRepository(db);

export const load: PageServerLoad = async ({ cookies }) => {
	const user = await userRepo.getUserById(cookies.get('sessionid') ?? '');
	return { user };
};

export const actions = {
	default: async ({ request, cookies }) => {

    const data = await request.formData();
    const publicKey = data.get('public_key');

    if (!publicKey) {
			return fail(400, { publicKey, missing: true });
		}
		
    const { data: responseData, error } = await registerUserHandler({
    	body: {
    		public_key: publicKey as string,
    		username: data.get('username') as string
    	}
    })

    if (error){
    	return fail(error.)	
    }
    
		const createdUser = await fetch(`${SERVICE_URL}/`)
		
	  let user = userRepo.createUser('', '', '');
	  
    const username = data.get('username');
    const password = data.get('password');
 	}
} satisfies Actions;
