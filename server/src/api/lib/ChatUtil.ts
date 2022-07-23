import Message, { IMessageDocument } from "../models/message";
import messageService from "../services/message.service";

export class ChatUtil {

    onMessage (data: IMessageDocument) {
        const msg = messageService.create(data);
        // send user mail of new message
    }
}