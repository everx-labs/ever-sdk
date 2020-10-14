/*
 * Copyright 2018-2020 TON DEV SOLUTIONS LTD.
 *
 * Licensed under the SOFTWARE EVALUATION License (the "License"); you may not use
 * this file except in compliance with the License.
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific TON DEV software governing permissions and
 * limitations under the License.
 *
 */

import apiJson from './api.json';
import {
    Api, ApiConst,
    ApiField,
    ApiFunction,
    ApiModule,
    ApiType, Code, parseApi,
} from './api';

const api: Api = (apiJson as any);

const MODULES_HEADER = `
import {ResponseHandler} from "./bin";

interface IClient {
    request(
        functionName: string,
        functionParams: any,
        responseHandler?: ResponseHandler
    ): Promise<any>;
}
`;

class TSCode extends Code {
    private responseHandlerParam: ApiField;
    private undefinedParam: ApiField;
    
    constructor(api: Api) {
        super(api);
        const module = api.modules[0];
        this.responseHandlerParam = {
            api,
            module,
            name: 'responseHandler',
            type: 'optional',
            inner: {
                api,
                module,
                type: 'ref',
                type_name: 'ResponseHandler',
            },
        };
        
        this.undefinedParam = {
            name: 'undefined',
            type: 'none',
            api,
            module,
        };
        
        
    }
    
    module(module: ApiModule): string {
        let ts = `// ${module.name} module\n\n`;
        
        for (const type of module.types) {
            ts += this.typeDef(type);
        }
        
        ts += `
export class ${Code.upperFirst(module.name)}Module {
    client: IClient;

    constructor(client: IClient) {
        this.client = client;
    }
`;
        
        for (const func of module.functions) {
            ts += this.functionImpl(func);
        }
        
        ts += '}\n\n';
        
        return ts;
    }
    
    field(field: ApiField, indent: string): string {
        const name = `${field.name}${field.type === 'optional' ? '?' : ''}`;
        const type = field.type === 'optional' ? field.inner : field;
        return `${indent}${name}: ${this.type(type, indent)}`;
    }
    
    fields(fields: ApiField[], indent: string): string {
        return fields.map(f => this.field(f, indent)).join(',\n');
    }
    
    typeVariant(variant: ApiField, indent: string): string {
        if (variant.type === 'struct') {
            const fields = this.fields(variant.fields, `${indent}    `);
            return `{\n${indent}    type: '${variant.name}',\n${fields}\n${indent}}`;
        } else {
            return this.type(variant, indent);
        }
    }
    
    constVariant(variant: ApiConst): string {
        switch (variant.type) {
        case 'string':
            return `'${variant.value}'`;
        case 'none':
            return variant.name;
        default:
            return variant.value;
        }
    }
    
    type(type: ApiType, indent: string): string {
        switch (type.type) {
        case 'none':
            return 'void';
        case 'ref':
            if (type.type_name === 'Value' || type.type_name === 'API') {
                return 'any';
            }
            const parts = type.type_name.split('.');
            return parts[parts.length - 1];
        case 'optional':
            return `${this.type(type.inner, indent)} | null`;
        case 'struct':
            const fields = type.fields;
            if (fields.length === 1 && fields[0].name === '') {
                return this.type(fields[0], indent);
            } else {
                return `{\n${this.fields(fields, `${indent}    `)}\n${indent}}`;
            }
        case 'enumOfTypes':
            return type.types.map(x => this.typeVariant(x, indent)).join(' | ');
        case 'array':
            return `${this.type(type.items, indent)}[]`;
        case 'enumOfConsts':
            return type.consts.map(c => this.constVariant(c)).join(' | ');
        case 'bigInt':
            return 'bigint';
        default:
            return type.type;
        }
    }
    
    typeDef(type: ApiField): string {
        return `export type ${type.name} = ${this.type(type, '')};\n\n`;
    }
    
    functionImpl(func: ApiFunction): string {
        const name = Code.camel(func.name.split('_'));
        const paramsDecl = [...func.params, this.responseHandlerParam]
            .map(p => `        ${this.field(p, '')}`).join('\n');
        const paramsCall = [
            ...(func.params.length > 0 ? func.params : [this.undefinedParam]),
            this.responseHandlerParam,
        ].map(p => `            ${p.name},`).join('\n');
        const resultDecl = this.type(func.result, '');
        const callName = `${func.module.name}.${func.name}`;
        
        return `
    ${name}(
${paramsDecl}
    ): Promise<${resultDecl}> {
        return this.client.request(
            '${callName}',
${paramsCall}
        );
    }\n`;
    }
    
    
    functionInterface(func: ApiFunction): string {
        const name = Code.camel(func.name.split('_'));
        const paramsDecl = [...func.params, this.responseHandlerParam]
            .map(p => `    ${p.name}: ${this.type(p, '')},`).join('\n');
        const resultDecl = this.type(func.result, '');
        return `
function ${name}(
${paramsDecl}
): Promise<${resultDecl}>;\n`;
    
    }
    
    modules(): string {
        const modules = api.modules.map(m => this.module(m)).join('');
        return `
${MODULES_HEADER}
${modules}
`;
    }
}

export const TS: Code = new TSCode(parseApi(apiJson));
