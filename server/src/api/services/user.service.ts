import l from "../../common/logger";
import { PasswordUtil } from "../lib/PasswordUtil";
import User, { IUserModel } from "../models/user";
import { Service } from "./service";

class UserService extends Service<IUserModel> {
  async deleteById(id: number): Promise<void> {
    l.info(`delete user with id ${id}`);
    await User.deleteById(id.toString());
  }
  
  async create (data: IUserModel): Promise<IUserModel> {
    l.info(`create user with data ${data}`);
    data.password = await PasswordUtil.hashPassword(data.password);
    const user = new User(data);
    const doc = (await user.save()) as IUserModel;
    return doc;
  }

  async getAll(): Promise<IUserModel[]> {
    l.info("fetch all users");
    const users = (await User.find(
      null,
      "-_id -__v"
    ).lean()) as IUserModel[];
    return users;
  }

  async getById(id: number): Promise<IUserModel> {
    l.info(`fetch user with id ${id}`);
    const user = (await User.findById(id).lean()) as IUserModel;
    return user;
  }

  async getByEmail (email: string): Promise<IUserModel> {
    l.info("fetch user by email, " + email);
    const user = (await User.findOne({
      email
    }).lean()) as IUserModel;
    return user;
  }

  async getByUsername (username: string): Promise<IUserModel> {
    l.info("fetch user by username: ", username);
    let user = (await User.findOne({ username }).lean()) as IUserModel;
    if (!user) {
      user = await this.getByEmail(username);
    }
    return user;
  }
}

export default new UserService();