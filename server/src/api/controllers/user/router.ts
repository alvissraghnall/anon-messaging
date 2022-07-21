import express from "express";
import controller from "./controller";
export default express
  .Router()
  .route("/user")
  .post(controller.create)