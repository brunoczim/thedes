type Table = {
    package: string,
    public: boolean | nil,
    name: string,
}

type NumberType =
    | 'uint8'
    | 'int8'
    | 'uint16'
    | 'int16'
    | 'uint32'
    | 'int32'
    | 'uint64'
    | 'int64'
    | 'uint128'
    | 'int128'
    | 'float32'
    | 'float64'

type BoolType = 'bool'

type StringType = 'string' | 'bytearray'

type NilType = 'nil'

type Type = NumberType | BoolType | StringType | NilType

type NumberValue = {
    type: NumberType,
    value: number,
}

type StringValue = {
    type: StringType,
    value: string,
}

type BoolValue = {
    type: BoolType,
    value: boolean,
}

type Value = NumberValue | StringValue | BoolValue

type Storage = {
    createTable: (self: Storage) -> Table
}
