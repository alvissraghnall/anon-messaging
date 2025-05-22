import { SERVICE_URL } from '$env/static/private';

type CreateUserPayload = {
  public_key: string;
  username: string;
};

type CreateUserSuccess = {
  user_id: string;
  username: string;
};

type CreateUserError = {
  errors: {
    [key: string]: {
      code: string;
      message: string;
    }[];
  };
};

export async function createUser(payload: CreateUserPayload): Promise<
  | { success: true; data: CreateUserSuccess }
  | { success: false; status: number; error?: CreateUserError }
> {
  try {
    const response = await fetch(`${SERVICE_URL}/api/users`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(payload),
    });

    if (response.ok) {
      const data = (await response.json()) as CreateUserSuccess;
      return { success: true, data };
    } else {
      const status = response.status;
      let error: CreateUserError | undefined;

      try {
        error = await response.json();
      } catch {
        // No JSON body, e.g. 409 or 500 with no body
      }

      return { success: false, status, error };
    }
  } catch (err) {
    return { success: false, status: 500 };
  }
}
