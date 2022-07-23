
export abstract class Service<T> {

    abstract getById (id: number): Promise<T>;

    abstract create (data: T): Promise<T>;

    abstract getAll (): Promise<T[]>;

    abstract deleteById (id: number): Promise<void>;
}