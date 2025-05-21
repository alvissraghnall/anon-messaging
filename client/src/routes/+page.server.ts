import { db, UserRepository } from '$lib/server/db';
import type { PageServerLoad, Actions } from './$types';
import { fail } from '@sveltejs/kit';
import { SERVICE_URL } from '$env/static/private';
//import { registerUserHandler } from '$lib/server/requests';
import { z } from 'zod';


type ValidationError = {
  code: string;
  message?: string;
  params: {
    [key: string]: any;
    path?: string;
  };
};

type ValidationErrorsKind =
  | { kind: "Field"; Field: ValidationError[] }
  | { kind: "Struct"; Struct: Record<string, unknown> }
  | { kind: "List"; List: Record<number, unknown> }; 

let userSchema = z.object({
	public_key: z.string().base64url(),
	username: z.string().trim().min(1),
})

let userRepo = new UserRepository(db);
 
/*
export const load: PageServerLoad = async ({ cookies }) => {
	const user = await userRepo.getUserById(cookies.get('sessionid') ?? '');
	return { user };
};
*/

export const load: PageServerLoad = async ({ fetch, params }) => {
	const res = await fetch(`https://eerip.onrender.com/api/users`); 
	const items = await res.json();
	console.log(items);

	return { users: items };
};

export const actions = {
	default: async ({ request, cookies }) => {
		console.log('there');
    
    const data = await request.formData();
    const publicKey = data.get('public_key');
    const username = data.get('username');
    const password = data.get('password');
    
	const formDataObject = Object.fromEntries(data);
    const userData = userSchema.safeParse(formDataObject);

    if (!userData.success) {
		  const errors = userData.error.errors.map((error) => {
		    return {
		      field: error.path[0],
		      message: error.message
		    };
		  });

		  return fail(400, { error: true, errors });
		}
		
    let { data: responseData, error } = await registerUserHandler({
    	body: {
    		public_key: publicKey as string,
    		username: username as string
    	}
    }) 

	console.log(responseData);
    console.log(error);

    if (error) {
		  if ((error as any).kind === "Field") {
		    const fieldErrors = (error as { kind: "Field"; Field: ValidationError[] }).Field;

		    const filteredErrors = fieldErrors.filter(
		      (err) => err.params?.path !== "public_key"
		    );

		    const formattedErrors = filteredErrors.reduce((acc, err) => {
		      const fieldName = err.params?.path || "unknown";
		      acc[fieldName] = err.message || err.code;
		      return acc;
		    }, {} as Record<string, string>);

		    return fail(400, { errors: formattedErrors });
		  }

		  return fail(500, { message: "Unexpected error structure" });
		}
    
		// const createdUser = await fetch(`${SERVICE_URL}/`)
		
	  // let user = userRepo.createUser('', '', '');
	  
 	}
} satisfies Actions;
