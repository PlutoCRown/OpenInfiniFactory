declare const Bun: {
    write(path: string, data: Uint8Array): void;
};

interface Serializable {
    serialize(writer: Writer): void;
    deserialize(reader: Reader): void;
}

// 序列化写入端，预分配 Int32 缓冲区
class Writer {
    static CAPACITY = 65536;
    private buffer: Int32Array;
    private length = 0;

    constructor(capacity = Writer.CAPACITY) {
        this.buffer = new Int32Array(capacity);
    }

    write(value: number): void {
        this.buffer[this.length++] = value | 0;
    }

    save_to(path: string): void {
        const used = this.buffer.subarray(0, this.length);
        Bun.write(path, new Uint8Array(used.buffer, used.byteOffset, used.byteLength));
    }
}

// 序列化读取端，顺序消费 Int32 缓冲区
class Reader {
    private index = 0;

    constructor(private buffer: Int32Array) { }

    read_number(): number {
        return this.buffer[this.index++];
    }
}

// 序列化工具
class Serializer {
    static get_reader(bytes: Uint8Array): Reader {
        const buffer = new Int32Array(bytes.buffer, bytes.byteOffset, bytes.byteLength / 4);
        return new Reader(buffer);
    }

    static get_writer(capacity = Writer.CAPACITY): Writer {
        return new Writer(capacity);
    }
}
