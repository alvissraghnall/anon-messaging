import Message, { IMessageModel } from "../models/message";

export class ChatUtil {

    onMessage (data: IMessageModel) {
        const message = new Message(data);
    }
}