import { db, UserRepository } from "$lib/server/db";
import { redirect } from "@sveltejs/kit";
import type { PageServerLoad } from "../$types";
import { zGetMessageHandlerResponse } from "$lib/server/requests/zod.gen";
import { getUserThreadsHandler } from "$lib/server/requests";
import { generalClient } from "$lib/server/client";

let userRepo = new UserRepository(db);

export const load: PageServerLoad = async ({ cookies }) => {
	const user = await userRepo.getUserById(cookies.get('user_id') ?? '');
	if(!user) {
	  redirect(308, "/");
	}
	const threads = await getUserThreadsHandler({
	  client: generalClient,
	  path: {
	    user_id: user.id
	  }
	});

	console.log(threads);
};
