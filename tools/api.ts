export type Api = {
    version: string,
    modules: ApiModule[],
}

export type ApiModule = {
    api: Api;
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
    ref_type?: ApiType,
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
    type: ApiTypeIs.EnumOfTypes
    enum_types: ApiField[]
}

export type ApiGeneric = {
    type: ApiTypeIs.Generic,
    generic_name: string,
    generic_args: ApiType[],
}

export type ApiType = {
    module: ApiModule,
} & (
    { type: ApiTypeIs.None } |
    { type: ApiTypeIs.Any } |
    { type: ApiTypeIs.Boolean } |
    { type: ApiTypeIs.String } |
    { type: ApiTypeIs.Number } |
    { type: ApiTypeIs.BigInt } |
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

function isFullName(name: string): boolean {
    return name.includes('.');
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
        }
    }
    const resolveRefs = (module: ApiModule, type: ApiType) => {
        switch (type.type) {
        case ApiTypeIs.Ref:
            const name = type.ref_name;
            if (!isFullName(name)) {
                const refType = types.get(`${module.name}.${name}`) ?? types.get(name);
                if (refType) {
                    type.ref_name = `${refType.module.name}.${refType.name}`;
                    type.ref_type = refType;
                }
            }
            break;
        case ApiTypeIs.Generic:
            for (const arg of type.generic_args) {
                resolveRefs(module, arg);
            }
            break;
        case ApiTypeIs.Array:
            resolveRefs(module, type.array_item);
            break;
        case ApiTypeIs.EnumOfTypes:
            for (const variant of type.enum_types) {
                resolveRefs(module, variant);
            }
            break;
        case ApiTypeIs.Optional:
            resolveRefs(module, type.optional_inner);
            break;
        case ApiTypeIs.Struct:
            for (const field of type.struct_fields) {
                resolveRefs(module, field);
            }
            break;
        }
    };

    const reduceFunc = (func: ApiFunction) => {
        for (const param of func.params) {
            resolveRefs(func.module, param);
        }
        if (func.result.type === ApiTypeIs.Generic && func.result.generic_name === 'ClientResult') {
            func.result = func.result.generic_args[0];
        }
        resolveRefs(func.module, func.result);
    };

    for (const module of api.modules) {
        module.api = api;
        for (const type of module.types) {
            resolveRefs(module, type);
        }
        for (const func of module.functions) {
            reduceFunc(func);
        }
    }
    return api;
}

export type ApiFunctionInfo = {
    params?: ApiField,
    hasResponseHandler: boolean,
}

export abstract class Code {
    readonly api: Api;

    constructor(api: Api) {
        this.api = api;
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

    getFunctionInfo(func: ApiFunction): ApiFunctionInfo {
        const info: ApiFunctionInfo = {
            hasResponseHandler: false,
        };
        for (const param of func.params) {
            if (param.type === ApiTypeIs.Generic && param.generic_name === 'Arc') {
                const arcArg = param.generic_args[0];
                const isContext = arcArg.type === 'Ref' && arcArg.ref_name === 'ClientContext';
                if (!isContext) {
                    info.hasResponseHandler = true;
                }
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

    static upperFirst(ident: string): string {
        return ident !== '' ? `${ident[0].toUpperCase()}${ident.substr(1)}` : '';
    }

    static lowerFirst(ident: string): string {
        return ident !== '' ? `${ident[0].toLowerCase()}${ident.substr(1)}` : '';
    }


    static pascal(words: string[]): string {
        return words.map(this.upperFirst).join('');
    }

    static camel(words: string[]): string {
        return this.lowerFirst(this.pascal(words));
    }
}
