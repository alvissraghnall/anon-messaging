import userService from "../../services/user.service";
import type { Request, Response, NextFunction } from "express";
import { PasswordUtil } from "../../lib/PasswordUtil";
import { Route, Get, Post, Controller, SuccessResponse } from "tsoa";

@Route("user")
class UserController extends Controller {
  
  @SuccessResponse("201", "Created")
  @Post()
  async create(req: Request, res: Response, next: NextFunction) {
    try {
      const data = req.body;
      const user = await userService.create(data);
      return res.status(201).location(`/api/v1/user`).end();
    
    } catch (err) {
      return next(err);
    }
  }

  async login (req: Request, res: Response, next: NextFunction) {
    try {
      const { username, password } = req.body;
      const findUser = await userService.getByUsername(username);
      if (!findUser) return res.status(400).json({
        message: "Invalid email/username provided."
      });
      if (!PasswordUtil.comparePassword(password, findUser.password)) {
        return res.status(400).json({ message: "The password is invalid" });
      }
      return res.status(200).location("/api/v1/user").end();

    } catch (error) {
      return next(error);
    }
  }
}

export default new UserController();
