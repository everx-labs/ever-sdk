import {
    ApiConst,
    ApiConstValueIs,
    ApiEnumOfConsts,
    ApiEnumOfTypes,
    ApiField,
    ApiFunction,
    ApiModule,
    ApiStruct,
    ApiType,
    ApiTypeIs,
    Code,
} from './api';

function summary(summary?: string): string {
    return summary ? ` – ${summary}` : '';
}

function description(description?: string): string {
    return description ? `${description}\n\n` : '';
}

function moduleFile(module: ApiModule): string {
    return `mod_${module.name}.md`;
}

function funcRef(func: ApiFunction, module?: ApiModule): string {
    return `[${func.name}](${module ? moduleFile(module) : ''}#${func.name})`;
}

function typeRef(t: ApiField, module?: ApiModule): string {
    return `[${t.name}](${module ? moduleFile(module) : ''}#${t.name})`;
}


export class Docs extends Code {
    readonly code: Code;
    
    constructor(code: Code) {
        super(code.api);
        this.code = code;
    }
    
    language(): string {
        return 'md';
    }
    
    typeDef(type: ApiField) {
        let md = '';
        md += `## ${type.name}\n`;
        md += `${description(type.description)}\n`;
        md += `\`\`\`${this.code.language()}\n${this.code.typeDef(type)}\`\`\`\n`;
        md += this.type(type, '');
        return md;
    }
    
    type(type: ApiType, indent: string): string {
        switch (type.type) {
        case ApiTypeIs.Ref:
            const refType = this.findType(type.ref_name);
            if (refType) {
                return this.type(refType, indent);
            }
            return `_${type.ref_name}_`;
        case ApiTypeIs.Optional:
            return `Optional value of:\n\n${this.type(type.optional_inner, indent)}`;
        case ApiTypeIs.Struct:
            return this.typeFields(type.struct_fields);
        case ApiTypeIs.EnumOfTypes:
            return this.enumOfTypes(type, indent);
        case ApiTypeIs.EnumOfConsts:
            return this.enumOfConsts(type);
        }
        return '';
    }
    
    private typeFields(fields: ApiField[]): string {
        let md = '';
        for (const field of fields) {
            md += this.field(field);
        }
        return md;
    }
    
    private enumOfTypes(type: ApiEnumOfTypes, indent: string) {
        let md = `Depends on value of the  \`type\` field.\n\n`;
        md += type.enum_types.map(v => this.typeVariant(v, indent)).join('\n');
        return md;
    }
    
    tupleFields(variant: ApiStruct, indent: string): ApiField[] {
        let fields = variant.struct_fields;
        if (fields.length !== 1 && fields[0].name !== '') {
            throw new Error(`Expected tuple with single value`);
        }
        const innerType = fields[0];
        if (innerType.type === ApiTypeIs.Ref) {
            const refType = this.findType(innerType.ref_name);
            if (refType && refType.type === ApiTypeIs.Struct) {
                return this.structFields(refType, indent);
            }
        } else if (innerType.type === ApiTypeIs.Struct) {
            return this.structFields(innerType, indent);
        }
        return [
            {
                ...innerType,
                name: 'value',
            },
        ];
    }
    
    structFields(variant: ApiStruct, indent: string): ApiField[] {
        const fields = variant.struct_fields;
        if (fields.length === 0) {
            return fields;
        }
        if (fields[0].name === '') {
            return this.tupleFields(variant, indent);
        }
        return fields;
    }
    
    typeVariant(variant: ApiField, indent: string): string {
        let ts = `When _type_ is _'${variant.name}'_\n\n`;
        if (variant.type === ApiTypeIs.Struct) {
            const fields = this.structFields(variant, indent);
            let fieldsDecl: string;
            if (fields.length === 0) {
                fieldsDecl = '';
            } else {
                fieldsDecl = `\n${this.typeFields(fields)}`;
            }
            ts += fieldsDecl;
        } else if (variant.type === ApiTypeIs.None) {
            ts += `\`${variant.name}\``;
        } else {
            ts += this.type(variant, indent);
        }
        return ts;
    }
    
    private enumOfConsts(type: ApiEnumOfConsts) {
        let md = `One of the following value:\n\n`;
        md += type.enum_consts.map(c => this.constVariant(c)).join('');
        return md;
    }
    
    
    constVariant(variant: ApiConst): string {
        let md = '- \`';
        switch (variant.type) {
        case ApiConstValueIs.None:
            md += variant.name;
            break;
        default:
            md += variant.value;
            break;
        }
        md += `\`${summary(variant.summary)}\n`;
        return md;
    }
    
    fieldType(type: ApiType): string {
        switch (type.type) {
        case ApiTypeIs.Ref:
            if (type.ref_name === 'Value') {
                return 'any';
            }
            const parts = type.ref_name.split('.');
            return parts.length === 2
                ? `[${parts[1]}](mod_${parts[0]}.md#${parts[1]})`
                : type.ref_name;
        case ApiTypeIs.Optional:
            return `${this.fieldType(type.optional_inner)}?`;
        case ApiTypeIs.Struct:
            return 'struct';
        case ApiTypeIs.EnumOfTypes:
            return 'enum';
        case ApiTypeIs.EnumOfConsts:
            return 'const';
        case ApiTypeIs.Array:
            return `${this.fieldType(type.array_item)}[]`;
        case ApiTypeIs.String:
            return 'string';
        case ApiTypeIs.Any:
            return 'any';
        case ApiTypeIs.Boolean:
            return 'boolean';
        case ApiTypeIs.Number:
            return 'number';
        case ApiTypeIs.Generic:
            return `${type.generic_name}<${this.fieldType(type.generic_args[0])}>`;
        case ApiTypeIs.BigInt:
            return 'bigint';
        case ApiTypeIs.None:
            return 'void';
        default:
            return '';
        }
    }
    
    field(field: ApiField): string {
        const opt = field.type === ApiTypeIs.Optional ? '?' : '';
        const type = field.type === ApiTypeIs.Optional ? field.optional_inner : field;
        const name = field.name !== '' ? `\`${field.name}\`${opt}: ` : '';
        return `- ${name}_${this.fieldType(type)}_${summary(field.summary)}\n`;
    }
    
    resolveRef(type: ApiType): ApiField | null {
        return type.type === ApiTypeIs.Ref ? this.findType(type.ref_name) : null;
    }
    
    functionInterface(func: ApiFunction) {
        let md = '';
        md += `## ${func.name}\n\n${description(func.description)}`;
        
        const funcInfo = this.getFunctionInfo(func);
        let code = '';
        if (funcInfo.params) {
            const params = this.resolveRef(funcInfo.params);
            if (params) {
                code += `${this.code.typeDef(params)}\n`;
            }
        }
        const result = this.resolveRef(func.result);
        if (result) {
            code += `${this.code.typeDef(result)}\n`;
        }
        code += this.code.functionInterface(func);
        md += `\`\`\`${this.code.language()}\n${code}\n\`\`\`\n`;
        
        if (funcInfo.params || funcInfo.hasResponseHandler) {
            md += '### Parameters\n';
            if (funcInfo.params) {
                md += this.type(funcInfo.params, '');
            }
            if (funcInfo.hasResponseHandler) {
                md += `- \`responseHandler\`?: _ResponseHandler_ – additional responses handler.`;
            }
        }
        md += '### Result\n\n';
        md += this.type(func.result, '');
        return md;
    }
    
    module(module: ApiModule) {
        let md = '';
        md += `# Module ${module.name}\n\n`;
        md += module.description;
        md += '\n## Functions\n';
        for (const func of module.functions) {
            md += `${funcRef(func)}${summary(func.summary)}\n\n`;
        }
        md += '## Types\n';
        for (const type of module.types) {
            md += `${typeRef(type)}${summary(type.summary)}\n\n`;
        }
        
        md += '\n# Functions\n';
        for (const func of module.functions) {
            md += this.functionInterface(func);
            md += '\n\n';
        }
        
        md += '# Types\n';
        for (const func of module.types) {
            md += this.typeDef(func);
            md += '\n\n';
        }
        
        return md;
    }
    
    modules(): string {
        let md = '';
        md += '# Modules\n';
        for (const module of this.api.modules) {
            md += `## [${module.name}](${moduleFile(module)})${summary(module.summary)}\n\n`;
            for (const func of module.functions) {
                md += `${funcRef(func, module)}${summary(func.summary)}\n\n`;
            }
        }
        
        return md;
    }
    
    functionImpl(func: ApiFunction): string {
        return this.functionInterface(func);
    }
}

