import l from "../../common/logger";
import Message, { IMessageDocument } from "../models/message";
import { Service } from "./service";

export abstract class AbstractMessageService extends Service<IMessageDocument> {}

export class MessageService extends AbstractMessageService {
    async getById(id: number): Promise<IMessageDocument> {
        l.info(`fetch Message with id ${id}`);
        const message = (await Message.findById(id).lean()) as IMessageDocument;
        return message;
    }

    async create(data: IMessageDocument): Promise<IMessageDocument> {
        l.info(`create Message with data ${data}`);
        const msg = new Message(data);
        const savedMsg = (await msg.save()) as IMessageDocument;
        return savedMsg;
    }

    async getAll(): Promise<IMessageDocument[]> {
        l.info("fetch all messages");
        const messages = (await Message.find(
        null,
        "-_id -__v"
        ).lean()) as IMessageDocument[];
        return messages;
    }

    async deleteById(id: number): Promise<void> {
        l.info(`delete Message with id ${id}`);
        await Message.deleteById(id.toString());    
    }

    
}

export default new MessageService();