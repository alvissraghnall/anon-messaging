import userService from "../../services/user.service";
import type { Request, Response, NextFunction } from "express";

export default class UserController {
  constructor(private req: Request, private res: Response, private next: NextFunction) {
  }
  
  async create() {
    try {
    const data = Pick<this.req.body, "username" | "email" | "password">;
    const user = await userService.create(data);
    return this.res.status(201).location(`/api/v1/user/${doc.id}`).end();
    
    } catch (err) {
      return this.next(err);
    }
  }
}

//export default new UserController();
