import mongoose, { Schema, Document } from "mongoose";

export interface IUserModel extends Document {
  username: string,
  email: string,
  password: string
}

const userSchema = new Schema({
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


const User = mongoose.model<IUserModel>('User', userSchema);
export default User;
