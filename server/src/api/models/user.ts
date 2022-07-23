import mongoose, { Schema, Document, Model } from "mongoose";

export interface IUserModel extends Document {
  username: string,
  email: string,
  password: string
}

export interface UserModel extends Model<IUserModel> {
  deleteById(id: string): void;
}

const userSchema = new Schema<IUserModel, UserModel>({
  username: {
    type: String,
    unique: true,
    required: true,
    min: 3
  },
  email: {
    type: String,
    unique: true,
    required: true,
    min: 10,
    max: 1024
  },
  password: {
    type: String,
    required: true
  }
}, {
  timestamps: true
});

userSchema.statics.deleteById = function (_id) {
  return this.deleteOne({
    _id
  });
}


const User = mongoose.model<IUserModel, UserModel>('User', userSchema);
export default User;
