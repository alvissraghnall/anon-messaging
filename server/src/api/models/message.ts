import mongoose, { Schema, Document, ObjectId } from "mongoose";

export interface IMessageModel extends Document {
  data: string,
  sender: ObjectId,
  receiver: ObjectId
}

const messageSchema = new Schema({
  data: String,
  
  sender: {
    type: Schema.Types.ObjectId,
    ref: "User"
  },
  
  receiver: {
    type: Schema.Types.ObjectId,
    ref: "User"
  },
}, {
  timestamps: true
});

const Message = mongoose.model<IMessageModel>("Model", messageSchema);

export default Message;