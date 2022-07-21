import type { Application } from "express";
import examplesRouter from "./api/controllers/example/router";
import userRouter from "./api/controllers/user/router";
export default function routes(app: Application): void {
  app.use("/api/v1/examples", examplesRouter);
  app.use("/api/v1/user", userRouter)
}