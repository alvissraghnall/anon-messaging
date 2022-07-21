<script>
    import FormView from "../views/Form.view.svelte";
    import { Navigate } from "svelte-router-spa";
    import { createForm } from "svelte-forms-lib";
    import * as yup from "yup";
    import yupPassword from "yup-password";
    import { userRegistration } from "../services/user.service";

    yupPassword(yup);

    const { form, errors, state, handleChange, handleSubmit } = createForm({
        initialValues: {
            username: "",
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
                .required(),
            username: yup
                .string()
                .minLength(3)
                .required()
        
        }),
        onSubmit: async (values) => {
            await userRegistration(values, isOpen, text)
                .catch(err => {
                    text = err.response.data.message ?? err.message;
                    isOpen = true;
            })
        }
  });
</script>

<FormView>
    <form class="space-y-6" on:submit={handleSubmit}>
      <h5 class="text-xl font-medium text-gray-900 dark:text-white">Sign up to our platform </h5>
    </form>
</FormView>
     