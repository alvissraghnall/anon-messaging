import express from "express";
import controller from "./controller";

const router = express.Router();

router
  .route("/")
  .post(controller.create);

router
  .post("/login", controller.login);

export default router;