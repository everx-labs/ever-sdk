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
    api: Api;
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

export type ApiConstValue =
    { type: 'none' } |
    { type: 'bool', value: string } |
    { type: 'string', value: string } |
    { type: 'number', value: string };


export type ApiConst = ApiConstValue & {
    name: string,
    summary?: string,
    description?: string,
}

export type ApiRef = { type: 'ref', type_name: string };
export type ApiOptional = { type: 'optional', inner: ApiType };
export type ApiArray = { type: 'array', items: ApiType };
export type ApiStruct = { type: 'struct', fields: ApiField[] };
export type ApiEnumOfConsts = { type: 'enumOfConsts', consts: ApiConst[] };
export type ApiEnumOfTypes = { type: 'enumOfTypes', types: ApiField[] };
export type ApiGeneric = { type: 'generic', type_name: string, args: ApiType[] };

export type ApiType = {
    api: Api,
    module: ApiModule,
} & (
    { type: 'none' } |
    { type: 'any' } |
    { type: 'boolean' } |
    { type: 'string' } |
    { type: 'number' } |
    { type: 'bigInt' } |
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
    for (const module of api.modules) {
        module.api = api;
        for (const type of module.types) {
            type.module = module;
            type.api = api;
        }
        for (const func of module.functions) {
            func.module = module;
            func.api = api;
        }
    }
    return api;
}

export abstract class Code {
    readonly api: Api;
    
    protected constructor(api: Api) {
        this.api = api;
    }
    
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
