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
    l.info(`fetch example with id ${id}`);
    const user = (await User.findById(id).lean()) as IUserModel;
    return user;
  }
}

export default new UserService();