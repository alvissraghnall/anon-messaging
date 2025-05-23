import { db, UserRepository } from '$lib/server/db';
import type { PageServerLoad, Actions } from './$types';
import { fail } from '@sveltejs/kit';
import { SERVICE_URL } from '$env/static/private';
import { registerUserHandler } from '$lib/server/requests';
import { z } from 'zod';
import { createUser } from '$lib/server/forward/create-user';

import { createClient } from '@hey-api/client-fetch';

const myClient = createClient({
	baseUrl: SERVICE_URL
});

type ValidationError = {
	code: string;
	message?: string;
	params: {
		max?: number;
		min?: number;
		value: any;
		path?: string;
	};
};

type ValidationErrorsKind =
	| { kind: 'Field'; Field: ValidationError[] }
	| { kind: 'Struct'; Struct: Record<string, unknown> }
	| { kind: 'List'; List: Record<number, unknown> };

const emptyStringToUndefined = z.literal('').transform(() => undefined);

export function asOptionalField<T extends z.ZodTypeAny>(schema: T) {
	return schema.optional().or(emptyStringToUndefined);
}

const userSchema = z.object({
	public_key: z
		.string({
			required_error: 'Public key is required',
			invalid_type_error: 'Public key must be a string'
		})
		.base64url({ message: 'Public key must be a valid base64url string' }),

	username: z
		.string({
			invalid_type_error: 'Username must be a string'
		})
		.trim()
		.min(3, { message: 'Username cannot be less than 3 chars' })
		.max(50, { message: 'Username cannot have over 50 chars' })
		.optional()
		.or(emptyStringToUndefined),

	password: z
		.string({
			required_error: 'Password is required',
			invalid_type_error: 'Password must be a string'
		})
		.min(8, { message: 'Password must be at least 8 characters long' })
});

let userRepo = new UserRepository(db);

export const load: PageServerLoad = async ({ cookies }) => {
	const user = await userRepo.getUserById(cookies.get('user_id') ?? '');
	if (!user) return null;
	return { id: user.id, username: user.username };
};

/*
export const load: PageServerLoad = async ({ fetch, params }) => {
	const res = await fetch(`https://eerip.onrender.com/api/users`);
	const items = await res.json();
	console.log(items);

	return { users: items };
};
*/

export const actions = {
	default: async ({ request, cookies }) => {
		console.log('there');

		const data = await request.formData();
		const publicKey = data.get('public_key');
		const username = data.get('username');
		const password = data.get('password');

		const formDataObject = Object.fromEntries(data);
		console.log(formDataObject);
		const userData = userSchema.safeParse(formDataObject);

		if (!userData.success) {
			console.log(userData.error);
			const errors = userData.error.errors.map((error) => {
				return {
					field: error.path[0],
					message: error.message
				};
			});

			return fail(400, { error: true, errors });
		}
		console.log(userData);

		let { data: responseData, error } = await registerUserHandler({
			body: {
				public_key: userData.data.public_key,
				username: userData.data.username
			},
			url: '/api/users',
			headers: {
				'Content-Type': 'application/json'
			},
			client: myClient
		});

		let { user_id, username: resUsername } = await responseData;

		console.log(user_id, error);
		/*		
		const result = await createUser({ public_key: publicKey as string, username: username as string });

		console.log(result);
		console.log(result.error);

		if (error) {
			// let error = result.error;
			if ((error as any).kind === 'Field') {
				const fieldErrors = (error as { kind: 'Field'; Field: ValidationError[] }).Field;

				const filteredErrors = fieldErrors.filter((err) => err.params?.path !== 'public_key');

				const formattedErrors = filteredErrors.reduce(
					(acc, err) => {
						const fieldName = err.params?.path || 'unknown';
						acc[fieldName] = err.message || err.code;
						return acc;
					},
					{} as Record<string, string>
				);

				return fail(400, { errors: formattedErrors });
			}

			return fail(500, { message: 'Unexpected error structure' });
		}
*/

		if (error) {
			const fieldErrors = error as Record<string, Array<ValidationError>>;

			const filteredErrors = Object.entries(fieldErrors).reduce(
				(acc, [field, errors]) => {
					if (field !== 'public_key') {
						acc[field] = errors;
					}
					return acc;
				},
				{} as typeof fieldErrors
			);

			return fail(400, { errors: filteredErrors });
		}

		let user = await userRepo.createUser(user_id, resUsername, userData.data.password);

		cookies.set('user_id', user_id, { path: '/' });

		return { success: true, id: user_id, username: resUsername };
	}
} satisfies Actions;
