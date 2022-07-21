import userService from "../../services/user.service";
import type { Request, Response, NextFunction } from "express";

class UserController {
  constructor() {
  }
  
  async create(req: Request, res: Response, next: NextFunction) {
    try {
    const data = req.body;
    const user = await userService.create(data);
    return res.status(201).location(`/api/v1/user`).end();
    
    } catch (err) {
      return next(err);
    }
  }
}

export default new UserController();
