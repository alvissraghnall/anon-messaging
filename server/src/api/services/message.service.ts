import { IMessageModel } from "../models/message";
import { Service } from "./service";

export abstract class AbstractMessageService extends Service<IMessageModel> {}

export class MessageService extends AbstractMessageService {
    getById(id: number): Promise<IMessageModel> {
        throw new Error("Method not implemented.");
    }
    create(data: IMessageModel): Promise<IMessageModel> {
        throw new Error("Method not implemented.");
    }
    getAll(): Promise<IMessageModel[]> {
        throw new Error("Method not implemented.");
    }
    deleteById(id: number): Promise<void> {
        throw new Error("Method not implemented.");
    }

    
}