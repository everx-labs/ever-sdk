export type Api = {
    version: string,
    modules: ApiModule[],
}

export type ApiModule = {
    api: Api,
    name: string,
    summary?: string,
    description?: string,
    types: ApiField[],
    functions: ApiFunction[],
}

export type ApiFunction = {
    module: ApiModule,
    name: string,
    summary?: string,
    description?: string,
    params: ApiField[],
    result: ApiType,
    errors?: ApiError[],
}

export type ApiField = ApiType & {
    name: string,
    summary?: string,
    description?: string,
}

export enum ApiConstValueIs {
    None = 'None',
    Bool = 'Bool',
    String = 'String',
    Number = 'Number',
}

export type ApiConstValue =
    { type: ApiConstValueIs.None } |
    { type: ApiConstValueIs.Bool, value: string } |
    { type: ApiConstValueIs.String, value: string } |
    { type: ApiConstValueIs.Number, value: string };


export type ApiConst = ApiConstValue & {
    name: string,
    summary?: string,
    description?: string,
}

export enum ApiTypeIs {
    Ref = 'Ref',
    None = 'None',
    Any = 'Any',
    Boolean = 'Boolean',
    String = 'String',
    Number = 'Number',
    BigInt = 'BigInt',
    Optional = 'Optional',
    Array = 'Array',
    Struct = 'Struct',
    EnumOfConsts = 'EnumOfConsts',
    EnumOfTypes = 'EnumOfTypes',
    Generic = 'Generic',
}


export type ApiRef = {
    type: ApiTypeIs.Ref,
    ref_name: string,
}

export type ApiOptional = {
    type: ApiTypeIs.Optional,
    optional_inner: ApiType
}

export type ApiArray = {
    type: ApiTypeIs.Array,
    array_item: ApiType
}

export type ApiStruct = {
    type: ApiTypeIs.Struct,
    struct_fields: ApiField[]
}

export type ApiEnumOfConsts = {
    type: ApiTypeIs.EnumOfConsts,
    enum_consts: ApiConst[]
}

export type ApiEnumOfTypes = {
    type: ApiTypeIs.EnumOfTypes,
    enum_types: ApiField[],
}

export type ApiGeneric = {
    type: ApiTypeIs.Generic,
    generic_name: string,
    generic_args: ApiType[],
}

export enum ApiNumberType {
    UInt = 'UInt',
    Int = 'Int',
    Float = 'Float',
}

export type ApiNumber = {
    type: ApiTypeIs.Number,
    number_type: ApiNumberType,
    number_size: number,
}

export type ApiBigInt = {
    type: ApiTypeIs.BigInt,
    number_type: ApiNumberType,
    number_size: number,
}

export type ApiType = {
    module: ApiModule,
} & (
    { type: ApiTypeIs.None } |
    { type: ApiTypeIs.Any } |
    { type: ApiTypeIs.Boolean } |
    { type: ApiTypeIs.String } |
    ApiNumber |
    ApiBigInt |
    ApiRef |
    ApiOptional |
    ApiArray |
    ApiStruct |
    ApiEnumOfConsts |
    ApiEnumOfTypes |
    ApiGeneric);


export type ApiError = {
    code: number,
    message: string,
    data?: any,
}

export function parseApi(json: any): Api {
    const api: Api = json;
    const types = new Map<string, ApiField>();
    for (const module of api.modules) {
        module.api = api;
        for (const type of module.types) {
            type.module = module;
            types.set(`${module.name}.${type.name}`, type);
            types.set(type.name, type);
        }
        for (const func of module.functions) {
            func.module = module;
            if (func.result.type === ApiTypeIs.Generic && func.result.generic_name === 'ClientResult') {
                func.result = func.result.generic_args[0];
            }
        }
    }
    return api;
}

export type ApiFunctionInfo = {
    params?: ApiField,
    hasResponseHandler: boolean,
    appObject?: ApiModule,
}

export abstract class Code {
    readonly api: Api;

    constructor(api: Api) {
        this.api = api;
    }

    static upperFirst(ident: string): string {
        return ident !== '' ? `${ident[0].toUpperCase()}${ident.substr(1)}` : '';
    }

    static lowerFirst(ident: string): string {
        return ident !== '' ? `${ident[0].toLowerCase()}${ident.substr(1)}` : '';
    }

    static pascal(words: string[]): string {
        return words.map(this.upperFirst).join('');
    }

    static pascalToSnake(name: string) {
        let snake = '';
        for (let i = 0; i < name.length; i += 1) {
            const lower = name[i].toLowerCase();
            if (lower !== lower.toUpperCase() && name[i] !== lower && snake !== '') {
                snake += '_';
            }
            snake += lower;
        }
        return snake;
    }

    static camel(words: string[]): string {
        return this.lowerFirst(this.pascal(words));
    }

    findType(name: string): ApiField | null {
        for (const module of this.api.modules) {
            for (const type of module.types) {
                if (name === `${module.name}.${type.name}`) {
                    return type;
                }
            }
        }
        return null;
    }


    getAppObject(source: ApiGeneric): ApiModule {
        const requiredRefEnum = (type: ApiType, name: string): ApiEnumOfTypes => {
            const refType = type.type === ApiTypeIs.Ref ? this.findType(type.ref_name) : null;
            if (!refType) {
                throw new Error(`${name} type of an AppObject isn't registered in api.`)
            }
            if (refType.type !== ApiTypeIs.EnumOfTypes) {
                throw new Error(`${name} type must be an enum.`)
            }
            return refType;
        };
        const paramsEnum = requiredRefEnum(source.generic_args[0], 'ParamsOf');
        const resultEnum = requiredRefEnum(source.generic_args[1], 'ResultOf');
        const obj: ApiModule = {
            api: this.api,
            name: source.generic_args[0].type === ApiTypeIs.Ref
                ? source.generic_args[0].ref_name.split('.')[1].substr('ParamsOf'.length)
                : '',
            types: [],
            functions: [],
        };
        for (const params of paramsEnum.enum_types) {
            const result = resultEnum.enum_types.find(x => x.name === params.name);
            const functionParams: ApiField[] = [];
            if (params.type === ApiTypeIs.Struct && params.struct_fields.length > 0) {
                const paramsTypeName = `ParamsOf${obj.name}${params.name}`;
                obj.types.push({
                    ...params,
                    module: obj,
                    name: paramsTypeName,
                });
                functionParams.push({
                    module: obj,
                    name: 'params',
                    type: ApiTypeIs.Ref,
                    ref_name: paramsTypeName,
                })
            }

            const resultTypeName = `ResultOf${obj.name}${params.name}`;
            if (result && result.type === ApiTypeIs.Struct && result.struct_fields.length > 0) {
                obj.types.push({
                    ...result,
                    module: obj,
                    name: resultTypeName,
                });
            }
            obj.functions.push({
                module: obj,
                name: Code.pascalToSnake(params.name),
                params: functionParams,
                result: {
                    type: ApiTypeIs.Ref, module: obj, ref_name: result ? resultTypeName : ''
                },
            });
        }
        return obj;
    }

    getFunctionInfo(func: ApiFunction): ApiFunctionInfo {
        const info: ApiFunctionInfo = {
            hasResponseHandler: false,
        };
        for (const param of func.params) {
            if (param.type === ApiTypeIs.Generic && param.generic_name === 'Arc') {
                const arcArg = param.generic_args[0];
                if (arcArg.type === 'Ref' && arcArg.ref_name === 'ClientContext') {
                    // skip context parameter
                } else if (arcArg.type === 'Ref' && arcArg.ref_name === 'Request') {
                    info.hasResponseHandler = true;
                }
            }
            if (param.type === ApiTypeIs.Generic && param.generic_name === 'AppObject') {
                info.appObject = this.getAppObject(param);
            } else if (param.name === 'params') {
                info.params = param;
            }
        }
        return info;
    }

    abstract language(): string;

    abstract module(module: ApiModule): string;

    abstract field(field: ApiField, indent: string): string;

    abstract typeVariant(variant: ApiField, indent: string): string;

    abstract constVariant(variant: ApiConst): string;

    abstract type(type: ApiType, indent: string): string;

    abstract typeDef(type: ApiField): string;

    abstract functionImpl(func: ApiFunction): string;

    abstract functionInterface(func: ApiFunction): string;

    abstract modules(): string;
}
