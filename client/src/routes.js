import Home from "./pages/Home.svelte";
import Login from "./pages/Login.svelte"
import Register from "./pages/Register.svelte"
import PrivacyPolicy from "./pages/PrivacyPolicy.svelte"

export const routes = [
  {
    name: "/",
    component: Home
  }, 
  {
    name: "login",
    component: Login
  },
  {
    name: "register",
    component: Register
  },
  {
    name: "privacy-policy",
    component: PrivacyPolicy
  }
];
