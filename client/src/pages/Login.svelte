<script>
  import FormView from "../views/Form.view.svelte";
  import { Navigate } from "svelte-router-spa";
  import { createForm } from "svelte-forms-lib";
  import * as yup from "yup";
  import yupPassword from "yup-password";
  import { userLogin } from "../services/user.service";

  yupPassword(yup);

  const { form, errors, state, handleChange, handleSubmit } = createForm({
    initialValues: {
      password: "",
      email: ""
    },
    validationSchema: yup.object().shape({
      email: yup
        .string()
        .email()
        .required(),
      password: yup
        .string()
        .password()
        .minSymbols(0)
        .required()
    }),
    onSubmit: async (values) => {
      await userLogin(values, isOpen, text)
        .catch(err => {
          text = err.response.data.message ?? err.message;
          isOpen = true;
        })
    }
  });
  let text = "", isOpen = false;
  $: invalidEmailClasses = $errors.email ? "bg-red-50 border border-red-500 text-red-900 placeholder-red-700 text-sm rounded-lg focus:ring-red-500 focus:border-red-500 block w-full p-2.5 dark:bg-red-100 dark:border-red-400" : "";
  $: invalidPasswordClasses = $errors.password ? "bg-red-50 border border-red-500 text-red-900 placeholder-red-700 text-sm rounded-lg focus:ring-red-500 focus:border-red-500 block w-full p-2.5 dark:bg-red-100 dark:border-red-400" : "";
  $: validClasses = "bg-green-50 border border-green-500 text-green-900 placeholder-green-700 text-sm rounded-lg focus:ring-green-500 focus:border-green-500 block w-full p-2.5 dark:bg-green-100 dark:border-green-400";
</script>

<div id="login">
  <FormView>
    <form class="space-y-6" on:submit={handleSubmit}>
      <h5 class="text-xl font-medium text-gray-900 dark:text-white">Sign in to our platform</h5>
      <div class="mb-6">
        <label for="email" class="block mb-2 text-sm font-medium text-gray-900 dark:text-gray-300">Your email</label>
        <input 
          type="email" 
          name="email" 
          id="email" 
          class="bg-gray-50 border border-gray-300 text-gray-900 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block w-full p-2.5 dark:bg-gray-600 dark:border-gray-500 dark:placeholder-gray-400 dark:text-white {invalidEmailClasses}" 
          placeholder="name@company.com" 
          required
          on:change={handleChange}
          on:blur={handleChange}
          bind:value={$form.email}
        >
        {#if $errors.email}
          <small>{$errors.email}</small>
        {/if}
      </div>
      <div class="mb-6">
        <label for="password" class="block mb-2 text-sm font-medium text-gray-900 dark:text-gray-300">Your password</label>
        <input 
          type="password" 
          name="password" 
          id="password" 
          placeholder="••••••••" 
                                                                  class="bg-gray-50 border border-gray-300 text-gray-900 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block w-full p-2.5 dark:bg-gray-600 dark:border-gray-500 dark:placeholder-gray-400 dark:text-white {invalidPasswordClasses}" 
          required=""
          on:change={handleChange}
          on:blur={handleChange}
          bind:value={$form.password}
        >
        {#if $errors.password}
          <small>{$errors.password}</small>
        {/if}

      </div>
      <div class="flex items-start">
        <div class="flex items-start">
          <div class="flex items-center h-5">
            <input id="remember" type="checkbox" value="" class="w-4 h-4 bg-gray-50 rounded border border-gray-300 focus:ring-3 focus:ring-blue-300 dark:bg-gray-700 dark:border-gray-600 dark:focus:ring-blue-600 dark:ring-offset-gray-800" required="">
          </div>
          <label for="remember" class="ml-2 text-sm font-medium text-gray-900 dark:text-gray-300">Remember me</label>
        </div>
        <Navigate to="password/forgot" styles="ml-auto text-sm text-blue-700 hover:underline dark:text-blue-500">Lost Password?</Navigate>
      </div>
      <button type="submit" class="w-full text-white bg-blue-700 hover:bg-blue-800 focus:ring-4 focus:outline-none focus:ring-blue-300 font-medium rounded-lg text-sm px-5 py-2.5 text-center dark:bg-blue-600 dark:hover:bg-blue-700 dark:focus:ring-blue-800">Login to your account</button>
      <div class="text-sm font-medium text-gray-500 dark:text-gray-300">
        Not registered? <Navigate to="signup" styles="text-blue-700 hover:underline dark:text-blue-500">Create account</Navigate>
      </div>
    </form>
  </FormView>
</div>

<style>
  small {
    display: block;
    font-size: 12px;
    color: #ff6b6b;
    margin-top: 10px;
  }
</style>
