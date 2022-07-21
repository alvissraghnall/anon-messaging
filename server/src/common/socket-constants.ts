export enum ChatEvent {
    CONNECT = 'connect',
    DISCONNECT = 'disconnect',
    MESSAGE = "message"
}

export interface ChatMessage {
    author: string;
    message: string;
    receiver: string;
}