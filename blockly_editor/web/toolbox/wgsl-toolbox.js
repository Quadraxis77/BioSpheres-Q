// WGSL Toolbox Definition
// Consolidated toolbox for WGSL mode
// Requirements: 1.2, 1.3, 4.4, 9.1, 9.2, 10.3

const WgslToolbox = {
    mode: "wgsl",
    displayName: "WGSL",
    color: "#5C2E91",
    
    // Toolbox structure
    getToolbox: function() {
        return {
            kind: "categoryToolbox",
            contents: [
                {
                    kind: "category",
                    name: "Blocks",
                    colour: "#5C2E91",
                    contents: [
                        {
                            kind: "label",
                            text: "Toolbox cleared - ready to repopulate"
                        }
                    ]
                }
            ]
        };
    }
};
