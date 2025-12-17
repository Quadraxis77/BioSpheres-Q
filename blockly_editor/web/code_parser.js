// Code-to-Blocks Parser
// Parses code from multiple modes (Rust, WGSL, Bevy, Biospheres) and generates Blockly blocks

class MultiModeCodeParser {
    constructor(workspace) {
        this.workspace = workspace;
        this.blockIdCounter = 0;
        this.errors = [];
        this.references = new Map(); // Track cross-file references
        
        // Initialize mode-specific parsers
        this.rustParser = new RustParser(this);
        this.wgslParser = new WGSLParser(this);
        this.bevyParser = new BevyParser(this);
        this.biospheresParser = new BiospheresParser(this);
    }

    // Generate unique block ID
    generateBlockId() {
        return `block_${this.blockIdCounter++}`;
    }

    // Main parse function - detects mode and delegates to appropriate parser
    parse(code, mode = 'auto') {
        this.blockIdCounter = 0;
        this.errors = [];
        this.references.clear();
        
        try {
            // Auto-detect mode if not specified
            if (mode === 'auto') {
                mode = this.detectMode(code);
            }
            
            // Delegate to appropriate parser
            switch (mode) {
                case 'rust':
                    return this.rustParser.parse(code);
                case 'wgsl':
                    return this.wgslParser.parse(code);
                case 'bevy':
                    return this.bevyParser.parse(code);
                case 'biospheres':
                    return this.biospheresParser.parse(code);
                default:
                    this.addError(`Unknown mode: ${mode}`, 0, 0);
                    return [];
            }
        } catch (error) {
            this.addError(`Parse error: ${error.message}`, 0, 0);
            return [];
        }
    }
    
    // Detect code mode based on content
    detectMode(code) {
        // Check for WGSL-specific syntax
        if (code.includes('@compute') || code.includes('@vertex') || code.includes('@fragment') ||
            code.includes('var<storage>') || code.includes('var<uniform>')) {
            return 'wgsl';
        }
        
        // Check for Bevy-specific imports and types
        if (code.includes('use bevy::') || code.includes('Query<') || 
            code.includes('Commands') || code.includes('Res<') || code.includes('ResMut<')) {
            return 'bevy';
        }
        
        // Check for Biospheres-specific types
        if (code.includes('CellType') || code.includes('Genome') || 
            code.includes('AdhesionZone') || code.includes('SignalChannel') ||
            code.includes('emit_signal') || code.includes('contract_adhesions')) {
            return 'biospheres';
        }
        
        // Default to Rust
        return 'rust';
    }
    
    // Parse multiple files and preserve cross-file references
    parseMultipleFiles(files) {
        const results = new Map();
        
        // First pass: parse all files
        for (const [filename, code] of files.entries()) {
            const mode = this.detectModeFromFilename(filename);
            const blocks = this.parse(code, mode);
            results.set(filename, { mode, blocks });
        }
        
        // Second pass: detect and link cross-file references
        this.detectCrossFileReferences(results);
        
        return results;
    }
    
    // Detect mode from filename
    detectModeFromFilename(filename) {
        if (filename.endsWith('.wgsl')) return 'wgsl';
        if (filename.includes('system') || filename.includes('bevy')) return 'bevy';
        if (filename.includes('cell') || filename.includes('genome') || filename.includes('bio')) return 'biospheres';
        return 'rust';
    }
    
    // Detect cross-file references (imports, shader references, etc.)
    detectCrossFileReferences(fileResults) {
        for (const [filename, { blocks }] of fileResults.entries()) {
            this.scanBlocksForReferences(blocks, filename);
        }
    }
    
    // Recursively scan blocks for reference patterns
    scanBlocksForReferences(blocks, sourceFile) {
        for (const block of blocks) {
            // Check for use statements (imports)
            if (block.type === 'rust_use' || block.type === 'bevy_use') {
                const path = block.fields?.PATH || '';
                if (path.includes('::')) {
                    this.addReference(sourceFile, path, 'import');
                }
            }
            
            // Check for shader references in Bevy code
            if (block.type === 'bevy_shader_handle' || block.type === 'bevy_compute_pipeline') {
                const shaderPath = block.fields?.SHADER_PATH || '';
                if (shaderPath) {
                    this.addReference(sourceFile, shaderPath, 'shader');
                }
            }
            
            // Recursively check nested blocks
            if (block.values) {
                for (const valueBlock of Object.values(block.values)) {
                    if (valueBlock) {
                        this.scanBlocksForReferences([valueBlock], sourceFile);
                    }
                }
            }
            
            if (block.statements) {
                for (const stmtBlocks of Object.values(block.statements)) {
                    if (Array.isArray(stmtBlocks)) {
                        this.scanBlocksForReferences(stmtBlocks, sourceFile);
                    }
                }
            }
        }
    }
    
    // Add a cross-file reference
    addReference(sourceFile, targetPath, type) {
        if (!this.references.has(sourceFile)) {
            this.references.set(sourceFile, []);
        }
        this.references.get(sourceFile).push({ targetPath, type });
    }
    
    // Get all references for a file
    getReferences(filename) {
        return this.references.get(filename) || [];
    }
    
    // Add error with context
    addError(message, line, column, suggestion = null) {
        this.errors.push({
            message,
            line,
            column,
            suggestion,
            timestamp: Date.now()
        });
    }
    
    // Get all parsing errors
    getErrors() {
        return this.errors;
    }
    
    // Check if parsing had errors
    hasErrors() {
        return this.errors.length > 0;
    }
    
    // Convert parsed blocks to Blockly XML
    blocksToXml(blocks) {
        let xml = '<xml xmlns="https://developers.google.com/blockly/xml">\n';
        
        for (let block of blocks) {
            xml += this.blockToXml(block, 2);
        }
        
        xml += '</xml>';
        return xml;
    }

    // Convert single block to XML
    blockToXml(block, indent = 0) {
        const spaces = ' '.repeat(indent);
        let xml = `${spaces}<block type="${block.type}" id="${block.id}">\n`;
        
        // Add fields
        if (block.fields) {
            for (let [name, value] of Object.entries(block.fields)) {
                xml += `${spaces}  <field name="${name}">${this.escapeXml(value)}</field>\n`;
            }
        }
        
        // Add values
        if (block.values) {
            for (let [name, valueBlock] of Object.entries(block.values)) {
                if (valueBlock) {
                    xml += `${spaces}  <value name="${name}">\n`;
                    xml += this.blockToXml(valueBlock, indent + 4);
                    xml += `${spaces}  </value>\n`;
                }
            }
        }
        
        // Add statements
        if (block.statements) {
            for (let [name, stmtBlocks] of Object.entries(block.statements)) {
                if (stmtBlocks && stmtBlocks.length > 0) {
                    xml += `${spaces}  <statement name="${name}">\n`;
                    for (let i = 0; i < stmtBlocks.length; i++) {
                        xml += this.blockToXml(stmtBlocks[i], indent + 4);
                        if (i < stmtBlocks.length - 1) {
                            xml += `${spaces}    <next>\n`;
                        }
                    }
                    for (let i = 0; i < stmtBlocks.length - 1; i++) {
                        xml += `${spaces}    </next>\n`;
                    }
                    xml += `${spaces}  </statement>\n`;
                }
            }
        }
        
        xml += `${spaces}</block>\n`;
        return xml;
    }

    // Escape XML special characters
    escapeXml(text) {
        return String(text)
            .replace(/&/g, '&amp;')
            .replace(/</g, '&lt;')
            .replace(/>/g, '&gt;')
            .replace(/"/g, '&quot;')
            .replace(/'/g, '&apos;');
    }

    // Load blocks into workspace
    loadIntoWorkspace(code, mode = 'auto') {
        const blocks = this.parse(code, mode);
        const xml = this.blocksToXml(blocks);
        const xmlDom = Blockly.utils.xml.textToDom(xml);
        Blockly.Xml.clearWorkspaceAndLoadFromXml(xmlDom, this.workspace);
    }
    
    // Create a simple text block for unparsed content
    createTextBlock(text) {
        return {
            type: 'rust_var',
            id: this.generateBlockId(),
            fields: {
                NAME: text
            }
        };
    }
}


// ============================================================================
// Rust Parser - Handles general Rust code
// ============================================================================
class RustParser {
    constructor(parent) {
        this.parent = parent;
    }
    
    parse(code) {
        const blocks = [];
        
        try {
            // Parse top-level constructs
            blocks.push(...this.parseUseStatements(code));
            blocks.push(...this.parseFunctions(code));
            blocks.push(...this.parseImpls(code));
            blocks.push(...this.parseStructs(code));
        } catch (error) {
            this.parent.addError(`Rust parse error: ${error.message}`, 0, 0, 'Check Rust syntax');
        }
        
        return blocks;
    }
    
    // Parse use statements
    parseUseStatements(code) {
        const blocks = [];
        const useRegex = /use\s+([^;]+);/g;
        let match;
        
        while ((match = useRegex.exec(code)) !== null) {
            blocks.push({
                type: 'rust_use',
                id: this.parent.generateBlockId(),
                fields: {
                    PATH: match[1].trim()
                }
            });
        }
        
        return blocks;
    }

    // Parse function definitions
    parseFunctions(code) {
        const blocks = [];
        
        // Match fn main()
        const mainRegex = /fn\s+main\s*\(\s*\)\s*\{([^}]*(?:\{[^}]*\}[^}]*)*)\}/g;
        let match;
        
        while ((match = mainRegex.exec(code)) !== null) {
            blocks.push({
                type: 'rust_main',
                id: this.parent.generateBlockId(),
                statements: {
                    BODY: this.parseStatements(match[1])
                }
            });
        }
        
        // Match regular functions
        const fnRegex = /fn\s+(\w+)\s*\(([^)]*)\)(?:\s*->\s*([^{]+))?\s*\{([^}]*(?:\{[^}]*\}[^}]*)*)\}/g;
        
        while ((match = fnRegex.exec(code)) !== null) {
            if (match[1] === 'main') continue;
            
            blocks.push({
                type: 'rust_function',
                id: this.parent.generateBlockId(),
                fields: {
                    NAME: match[1]
                },
                values: {
                    PARAMS: match[2] ? this.parent.createTextBlock(match[2].trim()) : null,
                    RETURN_TYPE: match[3] ? this.parent.createTextBlock(`-> ${match[3].trim()}`) : null
                },
                statements: {
                    BODY: this.parseStatements(match[4])
                }
            });
        }
        
        return blocks;
    }


    // Parse impl blocks
    parseImpls(code) {
        const blocks = [];
        const implRegex = /impl\s+(\w+)\s*\{([^}]*(?:\{[^}]*\}[^}]*)*)\}/g;
        let match;
        
        while ((match = implRegex.exec(code)) !== null) {
            blocks.push({
                type: 'rust_impl',
                id: this.parent.generateBlockId(),
                fields: {
                    TYPE: match[1]
                },
                statements: {
                    METHODS: this.parseStatements(match[2])
                }
            });
        }
        
        return blocks;
    }

    // Parse structs
    parseStructs(code) {
        const blocks = [];
        const structRegex = /struct\s+(\w+)\s*\{([^}]+)\}/g;
        let match;
        
        while ((match = structRegex.exec(code)) !== null) {
            blocks.push({
                type: 'rust_struct',
                id: this.parent.generateBlockId(),
                fields: {
                    NAME: match[1]
                },
                values: {
                    FIELDS: this.parent.createTextBlock(match[2].trim())
                }
            });
        }
        
        return blocks;
    }

    // Parse statements within a block
    parseStatements(code) {
        const statements = [];
        const lines = code.split('\n').map(l => l.trim()).filter(l => l);
        
        for (let line of lines) {
            try {
                if (line.startsWith('let ')) {
                    const stmt = this.parseLetBinding(line);
                    if (stmt) statements.push(stmt);
                } else if (line.startsWith('return ')) {
                    statements.push(this.parseReturn(line));
                } else if (line.startsWith('println!')) {
                    statements.push(this.parsePrintln(line));
                } else if (line.endsWith(';')) {
                    statements.push({
                        type: 'rust_expr_stmt',
                        id: this.parent.generateBlockId(),
                        values: {
                            EXPR: this.parent.createTextBlock(line.slice(0, -1))
                        }
                    });
                }
            } catch (error) {
                this.parent.addError(`Statement parse error: ${error.message}`, 0, 0);
            }
        }
        
        return statements;
    }


    // Parse let binding
    parseLetBinding(line) {
        const mutMatch = line.match(/let\s+(mut\s+)?(\w+)(?:\s*:\s*([^=]+))?\s*=\s*(.+);/);
        if (mutMatch) {
            return {
                type: 'rust_let_binding',
                id: this.parent.generateBlockId(),
                fields: {
                    MUTABLE: mutMatch[1] ? 'TRUE' : 'FALSE',
                    NAME: mutMatch[2]
                },
                values: {
                    TYPE: mutMatch[3] ? this.parent.createTextBlock(`: ${mutMatch[3].trim()}`) : null,
                    VALUE: this.parent.createTextBlock(mutMatch[4].trim())
                }
            };
        }
        return null;
    }

    // Parse return statement
    parseReturn(line) {
        const match = line.match(/return\s+(.+);/);
        if (match) {
            return {
                type: 'rust_return',
                id: this.parent.generateBlockId(),
                values: {
                    VALUE: this.parent.createTextBlock(match[1].trim())
                }
            };
        }
        return null;
    }

    // Parse println! macro
    parsePrintln(line) {
        const match = line.match(/println!\((.+)\);/);
        if (match) {
            return {
                type: 'rust_println',
                id: this.parent.generateBlockId(),
                values: {
                    MESSAGE: this.parent.createTextBlock(match[1].trim())
                }
            };
        }
        return null;
    }
}


// ============================================================================
// WGSL Parser - Handles WGSL shader code
// ============================================================================
class WGSLParser {
    constructor(parent) {
        this.parent = parent;
    }
    
    parse(code) {
        const blocks = [];
        
        try {
            // Parse WGSL constructs
            blocks.push(...this.parseStructs(code));
            blocks.push(...this.parseFunctions(code));
            blocks.push(...this.parseVariables(code));
        } catch (error) {
            this.parent.addError(`WGSL parse error: ${error.message}`, 0, 0, 'Check WGSL syntax');
        }
        
        return blocks;
    }
    
    // Parse WGSL structs
    parseStructs(code) {
        const blocks = [];
        const structRegex = /struct\s+(\w+)\s*\{([^}]+)\}/g;
        let match;
        
        while ((match = structRegex.exec(code)) !== null) {
            blocks.push({
                type: 'wgsl_struct',
                id: this.parent.generateBlockId(),
                fields: {
                    NAME: match[1]
                },
                values: {
                    FIELDS: this.parent.createTextBlock(match[2].trim())
                }
            });
        }
        
        return blocks;
    }
    
    // Parse WGSL functions (including entry points)
    parseFunctions(code) {
        const blocks = [];
        
        // Match @compute, @vertex, @fragment entry points
        const entryRegex = /@(compute|vertex|fragment)(?:\s+@workgroup_size\(([^)]+)\))?\s+fn\s+(\w+)\(([^)]*)\)(?:\s*->\s*([^{]+))?\s*\{([^}]*(?:\{[^}]*\}[^}]*)*)\}/g;
        let match;
        
        while ((match = entryRegex.exec(code)) !== null) {
            blocks.push({
                type: `wgsl_${match[1]}_shader`,
                id: this.parent.generateBlockId(),
                fields: {
                    NAME: match[3],
                    WORKGROUP_SIZE: match[2] || ''
                },
                values: {
                    PARAMS: match[4] ? this.parent.createTextBlock(match[4].trim()) : null,
                    RETURN_TYPE: match[5] ? this.parent.createTextBlock(match[5].trim()) : null
                },
                statements: {
                    BODY: this.parseStatements(match[6])
                }
            });
        }
        
        // Match regular functions
        const fnRegex = /fn\s+(\w+)\(([^)]*)\)(?:\s*->\s*([^{]+))?\s*\{([^}]*(?:\{[^}]*\}[^}]*)*)\}/g;
        
        while ((match = fnRegex.exec(code)) !== null) {
            // Skip if already matched as entry point
            if (code.substring(Math.max(0, match.index - 50), match.index).includes('@')) {
                continue;
            }
            
            blocks.push({
                type: 'wgsl_function',
                id: this.parent.generateBlockId(),
                fields: {
                    NAME: match[1]
                },
                values: {
                    PARAMS: match[2] ? this.parent.createTextBlock(match[2].trim()) : null,
                    RETURN_TYPE: match[3] ? this.parent.createTextBlock(match[3].trim()) : null
                },
                statements: {
                    BODY: this.parseStatements(match[4])
                }
            });
        }
        
        return blocks;
    }

    
    // Parse WGSL variables (storage, uniform, etc.)
    parseVariables(code) {
        const blocks = [];
        const varRegex = /var<(storage|uniform|private|workgroup)(?:,\s*(\w+))?>(?:\s+@group\((\d+)\)\s+@binding\((\d+)\))?\s+(\w+)\s*:\s*([^;]+);/g;
        let match;
        
        while ((match = varRegex.exec(code)) !== null) {
            blocks.push({
                type: 'wgsl_var',
                id: this.parent.generateBlockId(),
                fields: {
                    STORAGE_CLASS: match[1],
                    ACCESS_MODE: match[2] || '',
                    GROUP: match[3] || '',
                    BINDING: match[4] || '',
                    NAME: match[5],
                    TYPE: match[6].trim()
                }
            });
        }
        
        return blocks;
    }
    
    // Parse WGSL statements
    parseStatements(code) {
        const statements = [];
        const lines = code.split('\n').map(l => l.trim()).filter(l => l);
        
        for (let line of lines) {
            try {
                if (line.startsWith('let ') || line.startsWith('var ')) {
                    statements.push(this.parseVarDecl(line));
                } else if (line.startsWith('return ')) {
                    statements.push(this.parseReturn(line));
                } else if (line.endsWith(';')) {
                    statements.push({
                        type: 'wgsl_expr_stmt',
                        id: this.parent.generateBlockId(),
                        values: {
                            EXPR: this.parent.createTextBlock(line.slice(0, -1))
                        }
                    });
                }
            } catch (error) {
                this.parent.addError(`WGSL statement parse error: ${error.message}`, 0, 0);
            }
        }
        
        return statements;
    }
    
    // Parse variable declaration
    parseVarDecl(line) {
        const match = line.match(/(let|var)\s+(\w+)(?:\s*:\s*([^=]+))?\s*=\s*(.+);/);
        if (match) {
            return {
                type: 'wgsl_var_decl',
                id: this.parent.generateBlockId(),
                fields: {
                    KIND: match[1],
                    NAME: match[2],
                    TYPE: match[3] ? match[3].trim() : ''
                },
                values: {
                    VALUE: this.parent.createTextBlock(match[4].trim())
                }
            };
        }
        return null;
    }
    
    // Parse return statement
    parseReturn(line) {
        const match = line.match(/return\s+(.+);/);
        if (match) {
            return {
                type: 'wgsl_return',
                id: this.parent.generateBlockId(),
                values: {
                    VALUE: this.parent.createTextBlock(match[1].trim())
                }
            };
        }
        return null;
    }
}


// ============================================================================
// Bevy Parser - Handles Bevy ECS system code
// ============================================================================
class BevyParser {
    constructor(parent) {
        this.parent = parent;
    }
    
    parse(code) {
        const blocks = [];
        
        try {
            // Parse Bevy-specific constructs
            blocks.push(...this.parseUseStatements(code));
            blocks.push(...this.parseSystems(code));
            blocks.push(...this.parseComponents(code));
            blocks.push(...this.parseResources(code));
        } catch (error) {
            this.parent.addError(`Bevy parse error: ${error.message}`, 0, 0, 'Check Bevy ECS syntax');
        }
        
        return blocks;
    }
    
    // Parse use statements
    parseUseStatements(code) {
        const blocks = [];
        const useRegex = /use\s+([^;]+);/g;
        let match;
        
        while ((match = useRegex.exec(code)) !== null) {
            blocks.push({
                type: 'bevy_use',
                id: this.parent.generateBlockId(),
                fields: {
                    PATH: match[1].trim()
                }
            });
        }
        
        return blocks;
    }
    
    // Parse Bevy systems
    parseSystems(code) {
        const blocks = [];
        
        // Match system functions with Query parameters
        const systemRegex = /(?:pub\s+)?fn\s+(\w+)\s*\(([^)]*(?:Query|Commands|Res|ResMut)[^)]*)\)(?:\s*->\s*([^{]+))?\s*\{([^}]*(?:\{[^}]*\}[^}]*)*)\}/g;
        let match;
        
        while ((match = systemRegex.exec(code)) !== null) {
            blocks.push({
                type: 'bevy_system',
                id: this.parent.generateBlockId(),
                fields: {
                    NAME: match[1]
                },
                values: {
                    PARAMS: this.parent.createTextBlock(match[2].trim()),
                    RETURN_TYPE: match[3] ? this.parent.createTextBlock(match[3].trim()) : null
                },
                statements: {
                    BODY: this.parseStatements(match[4])
                }
            });
        }
        
        return blocks;
    }
    
    // Parse Bevy components
    parseComponents(code) {
        const blocks = [];
        const componentRegex = /#\[derive\([^)]*Component[^)]*\)\]\s*(?:pub\s+)?struct\s+(\w+)\s*\{([^}]+)\}/g;
        let match;
        
        while ((match = componentRegex.exec(code)) !== null) {
            blocks.push({
                type: 'bevy_component',
                id: this.parent.generateBlockId(),
                fields: {
                    NAME: match[1]
                },
                values: {
                    FIELDS: this.parent.createTextBlock(match[2].trim())
                }
            });
        }
        
        return blocks;
    }
    
    // Parse Bevy resources
    parseResources(code) {
        const blocks = [];
        const resourceRegex = /#\[derive\([^)]*Resource[^)]*\)\]\s*(?:pub\s+)?struct\s+(\w+)\s*\{([^}]+)\}/g;
        let match;
        
        while ((match = resourceRegex.exec(code)) !== null) {
            blocks.push({
                type: 'bevy_resource',
                id: this.parent.generateBlockId(),
                fields: {
                    NAME: match[1]
                },
                values: {
                    FIELDS: this.parent.createTextBlock(match[2].trim())
                }
            });
        }
        
        return blocks;
    }
    
    // Parse statements (reuse Rust parser logic)
    parseStatements(code) {
        const statements = [];
        const lines = code.split('\n').map(l => l.trim()).filter(l => l);
        
        for (let line of lines) {
            try {
                if (line.endsWith(';')) {
                    statements.push({
                        type: 'bevy_expr_stmt',
                        id: this.parent.generateBlockId(),
                        values: {
                            EXPR: this.parent.createTextBlock(line.slice(0, -1))
                        }
                    });
                }
            } catch (error) {
                this.parent.addError(`Bevy statement parse error: ${error.message}`, 0, 0);
            }
        }
        
        return statements;
    }
}


// ============================================================================
// Biospheres Parser - Handles Biospheres cell biology code
// ============================================================================
class BiospheresParser {
    constructor(parent) {
        this.parent = parent;
    }
    
    parse(code) {
        const blocks = [];
        
        try {
            // Parse Biospheres-specific constructs
            blocks.push(...this.parseUseStatements(code));
            blocks.push(...this.parseCellTypes(code));
            blocks.push(...this.parseCellBehaviors(code));
            blocks.push(...this.parseGenomeOperations(code));
        } catch (error) {
            this.parent.addError(`Biospheres parse error: ${error.message}`, 0, 0, 'Check Biospheres syntax');
        }
        
        return blocks;
    }
    
    // Parse use statements
    parseUseStatements(code) {
        const blocks = [];
        const useRegex = /use\s+([^;]+);/g;
        let match;
        
        while ((match = useRegex.exec(code)) !== null) {
            blocks.push({
                type: 'bio_use',
                id: this.parent.generateBlockId(),
                fields: {
                    PATH: match[1].trim()
                }
            });
        }
        
        return blocks;
    }
    
    // Parse cell type components
    parseCellTypes(code) {
        const blocks = [];
        const cellTypeRegex = /#\[derive\([^)]*Component[^)]*\)\]\s*pub\s+struct\s+(\w+)\s*\{([^}]+)\}/g;
        let match;
        
        while ((match = cellTypeRegex.exec(code)) !== null) {
            // Check if it's a cell-related component
            const name = match[1];
            if (name.includes('Cell') || name.includes('Type') || code.substring(0, match.index).includes('cell')) {
                blocks.push({
                    type: 'bio_cell_type_component',
                    id: this.parent.generateBlockId(),
                    fields: {
                        NAME: name
                    },
                    values: {
                        FIELDS: this.parent.createTextBlock(match[2].trim())
                    }
                });
            }
        }
        
        return blocks;
    }
    
    // Parse cell behavior systems
    parseCellBehaviors(code) {
        const blocks = [];
        
        // Match functions that use cell-specific operations
        const behaviorRegex = /(?:pub\s+)?fn\s+(\w+)\s*\(([^)]*)\)(?:\s*->\s*([^{]+))?\s*\{([^}]*(?:\{[^}]*\}[^}]*)*)\}/g;
        let match;
        
        while ((match = behaviorRegex.exec(code)) !== null) {
            const body = match[4];
            
            // Check if function contains cell-specific operations
            if (body.includes('emit_signal') || body.includes('contract_adhesions') || 
                body.includes('apply_thrust') || body.includes('CellType') ||
                body.includes('Genome') || body.includes('AdhesionZone')) {
                
                blocks.push({
                    type: 'bio_cell_behavior_system',
                    id: this.parent.generateBlockId(),
                    fields: {
                        NAME: match[1]
                    },
                    values: {
                        PARAMS: this.parent.createTextBlock(match[2].trim())
                    },
                    statements: {
                        BODY: this.parseStatements(body)
                    }
                });
            }
        }
        
        return blocks;
    }
    
    // Parse genome operations
    parseGenomeOperations(code) {
        const blocks = [];
        
        // Look for genome-related function calls
        const genomeRegex = /(\w+)\.get_genome\(\)|inject_genome\([^)]+\)|get_mode\([^)]+\)/g;
        let match;
        
        while ((match = genomeRegex.exec(code)) !== null) {
            blocks.push({
                type: 'bio_genome_operation',
                id: this.parent.generateBlockId(),
                values: {
                    OPERATION: this.parent.createTextBlock(match[0])
                }
            });
        }
        
        return blocks;
    }
    
    // Parse statements with cell-specific operations
    parseStatements(code) {
        const statements = [];
        const lines = code.split('\n').map(l => l.trim()).filter(l => l);
        
        for (let line of lines) {
            try {
                // Parse cell-specific operations
                if (line.includes('emit_signal')) {
                    statements.push(this.parseEmitSignal(line));
                } else if (line.includes('contract_adhesions')) {
                    statements.push(this.parseContractAdhesions(line));
                } else if (line.includes('apply_thrust') || line.includes('forces.force')) {
                    statements.push(this.parseApplyThrust(line));
                } else if (line.endsWith(';')) {
                    statements.push({
                        type: 'bio_expr_stmt',
                        id: this.parent.generateBlockId(),
                        values: {
                            EXPR: this.parent.createTextBlock(line.slice(0, -1))
                        }
                    });
                }
            } catch (error) {
                this.parent.addError(`Biospheres statement parse error: ${error.message}`, 0, 0);
            }
        }
        
        return statements;
    }
    
    // Parse emit_signal operation
    parseEmitSignal(line) {
        const match = line.match(/emit_signal\(([^,]+),\s*SignalChannel::(\w+),\s*([^,]+),\s*([^)]+)\);/);
        if (match) {
            return {
                type: 'bio_emit_signal',
                id: this.parent.generateBlockId(),
                fields: {
                    CHANNEL: match[2]
                },
                values: {
                    ENTITY: this.parent.createTextBlock(match[1].trim()),
                    VALUE: this.parent.createTextBlock(match[3].trim()),
                    RANGE: this.parent.createTextBlock(match[4].trim())
                }
            };
        }
        return this.parent.createTextBlock(line);
    }
    
    // Parse contract_adhesions operation
    parseContractAdhesions(line) {
        const match = line.match(/contract_adhesions\(([^,]+),\s*AdhesionZone::(\w+),\s*([^,]+),\s*([^)]+)\);/);
        if (match) {
            return {
                type: 'bio_contract_adhesions',
                id: this.parent.generateBlockId(),
                fields: {
                    ZONE: match[2]
                },
                values: {
                    ENTITY: this.parent.createTextBlock(match[1].trim()),
                    PERCENT: this.parent.createTextBlock(match[3].trim()),
                    SPEED: this.parent.createTextBlock(match[4].trim())
                }
            };
        }
        return this.parent.createTextBlock(line);
    }
    
    // Parse apply_thrust operation
    parseApplyThrust(line) {
        const match = line.match(/forces\.force\s*\+=\s*(.+?)\s*\*\s*([^;]+);/);
        if (match) {
            return {
                type: 'bio_apply_thrust',
                id: this.parent.generateBlockId(),
                values: {
                    DIRECTION: this.parent.createTextBlock(match[1].trim()),
                    FORCE: this.parent.createTextBlock(match[2].trim())
                }
            };
        }
        return this.parent.createTextBlock(line);
    }
}

// Maintain backward compatibility with old class name
class RustCodeParser extends MultiModeCodeParser {}
