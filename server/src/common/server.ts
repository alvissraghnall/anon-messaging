import express from "express";
import { Application } from "express";
import path from "path";
import http from "http";
import os from "os";
import cookieParser from "cookie-parser";
import l from "./logger";
import morgan from "morgan";
import { IDatabase } from "./database";

import errorHandler from "../api/middlewares/error.handler";
import * as OpenApiValidator from "express-openapi-validator";


export default class ExpressServer {
  private app: Application;
  private readonly root: string;
  private readonly apiSpec: string;
  private readonly validateResponses: boolean;
  
  constructor() {
    this.app = express();
    this.root = path.normalize(__dirname + "/../..");
    this.app.set("appPath", root + "client");
    this.app.use(morgan("dev"));
    this.app.use(express.json({ limit: process.env.REQUEST_LIMIT || "100kb" }));
    this.app.use(
      express.urlencoded({
        extended: true,
        limit: process.env.REQUEST_LIMIT || "100kb",
      })
    );
    this.app.use(express.text({ limit: process.env.REQUEST_LIMIT || "100kb" }));
    this.app.use(cookieParser(process.env.SESSION_SECRET));
    this.app.use(express.static(`${root}/public`));

    this.apiSpec = path.join(__dirname, "api.yml");
    this.validateResponses = !!(
      process.env.OPENAPI_ENABLE_RESPONSE_VALIDATION &&
      process.env.OPENAPI_ENABLE_RESPONSE_VALIDATION.toLowerCase() === "true"
    );
    this.app.use(process.env.OPENAPI_SPEC || "/spec", express.static(apiSpec));
    this.app.use(
      OpenApiValidator.middleware({
        apiSpec,
        validateResponses,
        ignorePaths: /.*\/spec(\/|$)/,
      })
    );
  }

  router(routes: (app: Application) => void): ExpressServer {
    routes(this.app);
    this.app.use(errorHandler);
    return this;
  }

  database(db: IDatabase): ExpressServer {
    db.init();
    return this;
  }

  listen(port: number): Application {
    const welcome = (p: number) => (): void =>
      l.info(
        `up and running in ${
          process.env.NODE_ENV || "development"
        } @: ${os.hostname()} on port: ${p}}`
      );

    http.createServer(this.app).listen(port, welcome(port));

    return this.app;
  }
}
