import mongoose, { Schema, Document, ObjectId, Model } from "mongoose";

export interface IMessageDocument extends Document {
  data: string,
  sender: ObjectId,
  receiver: ObjectId
}

export interface IMessageModel extends Model<IMessageDocument> {
  deleteById (id: string): void;
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
  timestamps: true,
});

messageSchema.statics.deleteById = function (_id) {
  return this.deleteOne({
    _id
  });
}

const Message = mongoose.model<IMessageDocument, IMessageModel>("Model", messageSchema);

export default Message;