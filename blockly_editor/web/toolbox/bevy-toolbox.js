// Bevy Toolbox Definition
// Consolidated toolbox for Bevy mode
// Requirements: 1.3, 4.4, 9.1, 9.2, 10.4

const BevyToolbox = {
    mode: "bevy",
    displayName: "Bevy",
    color: "#4EC9B0",
    
    // Toolbox structure
    getToolbox: function() {
        return {
            kind: "categoryToolbox",
            contents: [
                {
                    kind: "category",
                    name: "Blocks",
                    colour: "#4EC9B0",
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
