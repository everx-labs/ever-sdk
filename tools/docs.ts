import {ApiConst, ApiField, ApiFunction, ApiModule, ApiType, ApiTypeIs, Code} from './api';

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
        md += `## ${type.name}\n\n${description(type.description)}`;
        md += this.fields(type);
        return md;
    }
    
    type(type: ApiType, indent: string): string {
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
            return `${this.type(type.optional_inner, indent)}?`;
        case ApiTypeIs.Struct:
            return '{}';
        case ApiTypeIs.EnumOfTypes:
            return 'A | B';
        case ApiTypeIs.EnumOfConsts:
            return '0 | 1';
        case ApiTypeIs.Array:
            return `${this.type(type.array_item, indent)}[]`;
        case ApiTypeIs.String:
            return 'string';
        case ApiTypeIs.Any:
            return 'any';
        case ApiTypeIs.Boolean:
            return 'boolean';
        case ApiTypeIs.Number:
            return 'number';
        case ApiTypeIs.Generic:
            return `${type.generic_name}<>`;
        case ApiTypeIs.BigInt:
            return 'bigint';
        case ApiTypeIs.None:
            return 'void';
        default:
            return '';
        }
        
    }
    
    field(field: ApiField): string {
        return field.type === ApiTypeIs.Optional
            ? `- \`${field.name}\`?: _${this.type(
                field.optional_inner,
                '',
            )}_${summary(field.summary)}\n`
            : `- \`${field.name}\`: _${this.type(field, '')}_${summary(field.summary)}\n`;
    }
    
    
    fields(type: ApiType) {
        let md = '';
        
        switch (type.type) {
        case ApiTypeIs.Ref:
            const refType = this.findType(type.ref_name);
            if (refType) {
                md += this.fields(refType);
            }
            break;
        case ApiTypeIs.Optional:
            md += this.fields(type.optional_inner);
            break;
        case ApiTypeIs.Struct:
            for (const f of type.struct_fields) {
                md += this.field(f);
            }
            break;
        }
        return md;
    }
    
    functionInterface(func: ApiFunction) {
        let md = '';
        md += `## ${func.name}\n\n${description(func.description)}`;
        
        md += `\`\`\`${this.code.language()}\n${this.code.functionInterface(func)}\n\`\`\`\n`;
        
        if (func.params.length > 0) {
            md += '### Parameters\n';
            md += this.fields(func.params[0]);
        }
        md += '### Result\n\n';
        md += this.fields(func.result);
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
    
    constVariant(_variant: ApiConst): string {
        return '';
    }
    
    functionImpl(func: ApiFunction): string {
        return this.functionInterface(func);
    }
    
    typeVariant(_variant: ApiField, _indent: string): string {
        return '';
    }
    
}

