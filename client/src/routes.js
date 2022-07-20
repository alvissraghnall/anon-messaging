import Home from "./pages/Home.svelte";
import Login from "./pages/Login.svelte"

export const routes = [
  {
    name: "/",
    component: Home
  }, 
  {
    name: "login",
    component: Login
  }
];
