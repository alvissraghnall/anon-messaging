import l from "../../common/logger";
import User, { IUserModel } from "../models/user";

class UserService {
  
  async create (data: IUserModel): Promise<IUserModel> {
    l.info(`create example with data ${data}`);
    const user = new User(data);
    const doc = (await user.save()) as IUserModel;
    return doc;
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
}

export default new UserService();