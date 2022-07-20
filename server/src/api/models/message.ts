import mongoose, { Schema, Document } from "mongoose";


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

const Message = mongoose.model("Model", messageSchema);

export default Message;