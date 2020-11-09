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

import {
    ApiConst,
    ApiConstValueIs,
    ApiEnumOfTypes,
    ApiField,
    ApiFunction,
    ApiFunctionInfo,
    ApiModule,
    ApiStruct,
    ApiType,
    ApiTypeIs,
    Code,
} from './api';

const MODULES_HEADER = `
import {ResponseHandler} from "./bin";

interface IClient {
    request(
        functionName: string,
        functionParams?: any,
        responseHandler?: ResponseHandler
    ): Promise<any>;
}
`;

function isValidIdentFirstChar(c: string): boolean {
    return c >= 'A' && c <= 'Z' || c >= 'a' && c <= 'z' || c === '_';
}

function isValidIdentChar(c: string): boolean {
    return isValidIdentFirstChar(c) || (c >= '0' && c <= '9');
}

function fixFieldName(name: string): string {
    let isValidIdent = name !== '' && isValidIdentFirstChar(name[0]);
    if (isValidIdent) {
        for (let i = 1; i < name.length; i += 1) {
            if (!isValidIdentChar(name[i])) {
                isValidIdent = false;
                break;
            }
        }
    }
    return isValidIdent ? name : `'${name.split('\'').join('\\\'')}'`;
}

function typeName(fullName: string) {
    const parts = fullName.split('.');
    return parts[parts.length - 1];
}

export class TSCode extends Code {
    language(): string {
        return 'ts';
    }

    module(module: ApiModule): string {
        let ts = `// ${module.name} module\n\n`;

        for (const type of module.types) {
            ts += `\nexport ${this.typeDef(type)}`;
            if (type.type === ApiTypeIs.EnumOfTypes) {
                ts += this.typeVariantConstructors(type.name, type);
            }
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
        const name = `${fixFieldName(field.name)}${field.type === ApiTypeIs.Optional ? '?' : ''}`;
        const type = field.type === ApiTypeIs.Optional ? field.optional_inner : field;
        return `${indent}${name}: ${this.type(type, indent)}`;
    }

    fields(fields: ApiField[], indent: string): string {
        return fields.map(f => this.field(f, indent)).join(',\n');
    }

    typeVariantStructFields(variant: ApiStruct, _indent: string): ApiField[] {
        const fields = variant.struct_fields;
        if (fields.length === 0) {
            return fields;
        }
        return fields;
    }

    typeVariant(variant: ApiField, indent: string): string {
        if (variant.type === ApiTypeIs.Ref) {
            return `({\n${indent}    type: '${variant.name}'\n${indent}} & ${typeName(variant.ref_name)})`;
        } else if (variant.type === ApiTypeIs.Struct) {
            const fields = this.typeVariantStructFields(variant, indent);
            let fieldsDecl: string;
            if (fields.length === 0) {
                fieldsDecl = '';
            } else {
                fieldsDecl = `\n${this.fields(fields, `${indent}    `)}`;
            }
            return `{\n${indent}    type: '${variant.name}'${fieldsDecl}\n${indent}}`;
        } else if (variant.type === ApiTypeIs.None) {
            return `'${variant.name}'`;
        } else {
            return this.type(variant, indent);
        }
    }

    constVariant(variant: ApiConst): string {
        switch (variant.type) {
        case ApiConstValueIs.String:
            return `'${variant.value}'`;
        case ApiConstValueIs.None:
            return `'${variant.name}'`;
        case ApiConstValueIs.Bool:
            return variant.value;
        case ApiConstValueIs.Number:
            return variant.value;
        default:
            return '';
        }
    }

    type(type: ApiType, indent: string): string {
        switch (type.type) {
        case ApiTypeIs.None:
            return 'void';
        case ApiTypeIs.Ref:
            if (type.ref_name === 'Value' || type.ref_name === 'API') {
                return 'any';
            }
            return typeName(type.ref_name);
        case ApiTypeIs.Optional:
            return `${this.type(type.optional_inner, indent)} | null`;
        case ApiTypeIs.Struct:
            const fields = type.struct_fields;
            return `{\n${this.fields(fields, `${indent}    `)}\n${indent}}`;
        case ApiTypeIs.EnumOfTypes:
            return type.enum_types.map(x => this.typeVariant(x, indent)).join(' | ');
        case ApiTypeIs.Array:
            return `${this.type(type.array_item, indent)}[]`;
        case ApiTypeIs.EnumOfConsts:
            return type.enum_consts.map(c => this.constVariant(c)).join(' | ');
        case ApiTypeIs.BigInt:
            return 'bigint';
        case ApiTypeIs.Any:
            return 'any';
        case ApiTypeIs.String:
            return 'string';
        case ApiTypeIs.Number:
            return 'number';
        case ApiTypeIs.Boolean:
            return 'boolean';
        default:
            return type.type;
        }
    }

    typeDef(type: ApiField): string {
        return `type ${type.name} = ${this.type(type, '')};\n`;
    }

    paramsDecls(paramsInfo: ApiFunctionInfo): string[] {
        const decls: string[] = [];
        if (paramsInfo.params) {
            decls.push(`${paramsInfo.params.name}: ${this.type(paramsInfo.params, '')}`);
        }
        if (paramsInfo.hasResponseHandler) {
            decls.push('responseHandler?: ResponseHandler');
        }
        return decls;
    }

    functionInterface(func: ApiFunction): string {
        const paramsInfo = this.getFunctionInfo(func);
        const paramsDecls = this.paramsDecls(paramsInfo);
        const paramsDecl = paramsDecls.length > 0 ? `\n${paramsDecls.map(p => `    ${p},`)
            .join('\n')}\n` : '';
        const resultDecl = this.type(func.result, '');
        return `function ${func.name}(${paramsDecl}): Promise<${resultDecl}>;`;
    }

    functionImpl(func: ApiFunction): string {
        const paramsInfo = this.getFunctionInfo(func);
        const paramsDecl = this.paramsDecls(paramsInfo).map(p => `${p}`).join(', ');
        const calls = [`'${func.module.name}.${func.name}'`];
        if (paramsInfo.params) {
            calls.push(`${paramsInfo.params.name}`);
        }
        if (paramsInfo.hasResponseHandler) {
            if (!paramsInfo.params) {
                calls.push('undefined');
            }
            calls.push('responseHandler');
        }
        return `
    ${func.name}(${paramsDecl}): Promise<${this.type(func.result, '')}> {
        return this.client.request(${calls.join(', ')});
    }\n`;
    }

    modules(): string {
        return `
${MODULES_HEADER}
${this.api.modules.map(m => this.module(m)).join('')}
`;
    }

    private typeVariantConstructors(enumName: string, type: ApiEnumOfTypes): string {
        let ts = '';
        for (const variant of type.enum_types) {
            let params = '';
            let properties = '';
            switch (variant.type) {
            case ApiTypeIs.Ref:
                params = `params: ${typeName(variant.ref_name)}`;
                properties = `        ...params,\n`;
                break;
            case ApiTypeIs.Struct:
                const fields = variant.struct_fields;
                for (const field of fields) {
                    if (params !== '') {
                        params += ', ';
                    }
                    params += `${this.field(field, '')}`;
                    properties += `        ${fixFieldName(field.name)},\n`;

                }
                break;
            }
            ts +=
                `\nexport function ${TSCode.lowerFirst(enumName)}${variant.name}(${params}): ${enumName} {\n`;
            ts += `    return {\n`;
            ts += `        type: '${variant.name}',\n`;
            ts += properties;
            ts += `    };\n`;
            ts += `}\n`;
        }
        return ts;
    }

}
