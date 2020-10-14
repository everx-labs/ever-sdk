import {Api, ApiField, ApiFunction, ApiModule, ApiType} from "./api";
import fs from 'fs';
import path from 'path';
import {TS} from './ts-code';

const api: Api = TS.api;

function typeMD(type: ApiField) {
    let md = '';
    md += `## ${type.name}\n\n${description(type.description)}`;
    md += fieldsMD(type);
    return md;
}

function findType(name: string): ApiField | null {
    for (const module of api.modules) {
        for (const type of module.types) {
            if (name === `${module.name}.${type.name}`) {
                return type;
            }
        }
    }
    return null;
}

function typeNameMD(type: ApiType): string {
    switch (type.type) {
    case "ref":
        if (type.type_name === 'Value') {
            return 'any';
        }
        const parts = type.type_name.split('.');
        return parts.length === 2
            ? `[${parts[1]}](mod_${parts[0]}.md#${parts[1]})`
            : type.type_name;
    case "optional":
        return `${typeNameMD(type.inner)}?`;
    case "struct":
        return '{}';
    case "enumOfTypes":
        return 'A | B';
    case "enumOfConsts":
        return '0 | 1';
    case "array":
        return `${typeNameMD(type.items)}[]`;
    case "string":
        return 'string';
    case "any":
        return 'any';
    case "boolean":
        return 'boolean';
    case "number":
        return 'number';
    case "generic":
        return `${type.type_name}<>`;
    case "bigInt":
        return 'bigint';
    case "none":
        return 'void';
    default:
        return '';
    }
    
}

function fieldMD(field: ApiField): string {
    return field.type === "optional"
        ? `- \`${field.name}\`?: _${typeNameMD(field.inner)}_${summary(field.summary)}\n`
        : `- \`${field.name}\`: _${typeNameMD(field)}_${summary(field.summary)}\n`;
}


function fieldsMD(type: ApiType) {
    let md = '';
    
    switch (type.type) {
    case "ref":
        const refType = findType(type.type_name);
        if (refType) {
            md += fieldsMD(refType);
        }
        break;
    case "optional":
        md += fieldsMD(type.inner);
        break;
    case "struct":
        for (const f of type.fields) {
            md += fieldMD(f);
        }
        break;
    }
    return md;
}

function functionMD(func: ApiFunction) {
    let md = '';
    md += `## ${func.name}\n\n${description(func.description)}`;
    
    md += `\`\`\`ts\n${TS.functionInterface(func)}\n\`\`\`\n`;
    
    if (func.params.length > 0) {
        md += '### Parameters\n';
        md += fieldsMD(func.params[0]);
    }
    md += '### Result\n\n';
    md += fieldsMD(func.result);
    return md;
}

function moduleMD(module: ApiModule) {
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
        md += functionMD(func);
        md += '\n\n';
    }
    
    md += '# Types\n';
    for (const func of module.types) {
        md += typeMD(func);
        md += '\n\n';
    }
    
    return md;
}

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

function modulesMD(): string {
    let md = '';
    md += '# Modules\n';
    for (const module of api.modules) {
        md += `## [${module.name}](${moduleFile(module)})${summary(module.summary)}\n\n`;
        for (const func of module.functions) {
            md += `${funcRef(func, module)}${summary(func.summary)}\n\n`;
        }
    }
    
    return md;
}

export function generateDocs() {
    const outDir = path.resolve(__dirname, '..', 'docs');
    fs.writeFileSync(path.resolve(outDir, 'modules.md'), modulesMD(), 'utf8');
    for (const module of api.modules) {
        fs.writeFileSync(
            path.resolve(outDir, `mod_${module.name}.md`),
            moduleMD(module),
            'utf8'
        );
    }
}
