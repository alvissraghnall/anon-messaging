import l from "../../common/logger";
import User, { IUserModel } from "../models/user";

class UserService {
  
  async create (data: IUserModel): Promise<IUserModel> {
    l.info(`create example with data ${data}`);
    const user = new User(data);
    const doc = (await example.save()) as IUserModel;
    return doc;
  }
}

export default new UserService();