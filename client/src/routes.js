import Home from "./pages/Home.svelte";
import Login from "./pages/Login.svelte"
import Register from "./pages/Register.svelte"

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
  }
];
